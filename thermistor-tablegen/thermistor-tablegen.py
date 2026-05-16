#!/usr/bin/env python3
"""
Generate a Kalico/Klipper adc_temperature table for the stock Elegoo hotend
thermistor when the board has a fixed resistor in parallel with the thermistor.

Kalico/Danger-Klipper [adc_temperature] documentation:
  https://dangerklipper.io/Config_Reference.html#adc_temperature

Why this exists:
  The Elegoo stock hotend thermistor is treated as a 100K NTC with beta 4300 in
  the stock configuration. Some Centauri Carbon boards appear to place a 100K
  resistor in parallel with that thermistor. That parallel resistor compresses
  the measured resistance, especially at low temperatures, so using the normal
  beta thermistor config directly will read incorrectly.

  Kalico's thermistor inline_resistor option is not the right correction for a
  parallel shunt. In Kalico/Klipper-style thermistor math, inline_resistor is
  subtracted from the measured divider resistance before converting to
  temperature, which models a resistor in series with the thermistor path. A
  fixed parallel resistor must instead be folded into the resistance table.

The math:
  1. Convert Celsius to Kelvin:

       T_K = T_C + 273.15
       T0_K = reference_temp_C + 273.15

  2. Use the beta equation for the bare NTC thermistor:

       R_ntc(T) = R0 * exp(beta * (1 / T_K - 1 / T0_K))

     With the defaults, R0 is 100000 ohms at 25 C and beta is 4300.

  3. Combine the NTC with the fixed parallel resistor:

       R_table(T) = (R_ntc(T) * R_parallel) / (R_ntc(T) + R_parallel)

     R_table is what the MCU actually sees in the voltage divider, so it is
     what belongs in an [adc_temperature] resistance table.

  4. Kalico then converts that resistance through the normal divider model:

       adc_fraction = R_table / (pullup_resistor + R_table)

     The generated config leaves pullup_resistor as a heater setting because it
     is part of the analog input circuit, not part of the temperature table.
     No ADC voltage is needed for this generator. In the normal Klipper/Kalico
     thermistor path, the MCU reports a normalized ADC fraction. For resistance
     tables, Kalico combines that fraction with pullup_resistor to recover
     resistance, so the supply/reference voltage cancels out.

Sample count and interpolation:
  [adc_temperature] uses linear interpolation between table points. More points
  usually reduce interpolation error because this resistance curve is nonlinear.
  The default is 33 samples from 0 C through 320 C, matching the earlier
  generated printer.cfg-elegoon table at 10 C spacing. You can increase
  --samples or specify --step for denser tables. Very dense tables are useful
  for accuracy, but they also make printer.cfg longer and harder to inspect.
"""

from __future__ import annotations

import argparse
import math
import shlex
import sys
import textwrap
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


DEFAULT_OUTPUT = "printer.cfg-elegoon"
DEFAULT_SENSOR_NAME = "elegoo_100k_b4300_parallel_100k"
KALICO_ADC_TEMPERATURE_DOC_URL = "https://dangerklipper.io/Config_Reference.html#adc_temperature"


class VerboseHelpFormatter(argparse.ArgumentDefaultsHelpFormatter, argparse.RawDescriptionHelpFormatter):
    """Preserve help paragraphs while still showing argparse defaults."""

    def _get_help_string(self, action: argparse.Action) -> str:
        if action.default is None or action.default is argparse.SUPPRESS:
            return action.help
        return super()._get_help_string(action)


@dataclass(frozen=True)
class ThermistorModel:
    """Electrical model used to build the compensated resistance table."""

    r25: float = 100000.0
    beta: float = 4300.0
    reference_temp: float = 25.0
    parallel_resistor: float = 100000.0
    pullup_resistor: float = 4700.0

    def ntc_resistance(self, temp_c: float) -> float:
        """Return bare NTC resistance in ohms at temp_c using the beta equation."""
        temp_k = celsius_to_kelvin(temp_c)
        reference_k = celsius_to_kelvin(self.reference_temp)
        return self.r25 * math.exp(self.beta * ((1.0 / temp_k) - (1.0 / reference_k)))

    def table_resistance(self, temp_c: float) -> float:
        """Return the effective thermistor resistance with the parallel shunt."""
        ntc = self.ntc_resistance(temp_c)
        return parallel_resistance(ntc, self.parallel_resistor)

    def adc_fraction(self, temp_c: float) -> float:
        """Return the voltage-divider ADC fraction implied by the table resistance."""
        resistance = self.table_resistance(temp_c)
        return resistance / (self.pullup_resistor + resistance)

@dataclass(frozen=True)
class TablePoint:
    temp_c: float
    ntc_ohms: float
    table_ohms: float
    adc_fraction: float


@dataclass(frozen=True)
class InterpolationReport:
    max_error_c: float
    at_temp_c: float
    estimated_temp_c: float
    average_error_c: float
    dense_points: int


def celsius_to_kelvin(temp_c: float) -> float:
    kelvin = temp_c + 273.15
    if kelvin <= 0.0:
        raise ValueError(f"temperature {temp_c:g} C is at or below absolute zero")
    return kelvin


def parallel_resistance(a_ohms: float, b_ohms: float) -> float:
    return (a_ohms * b_ohms) / (a_ohms + b_ohms)


def format_float(value: float, decimals: int | None = None) -> str:
    if decimals is not None:
        return f"{value:.{decimals}f}"
    if math.isclose(value, round(value), abs_tol=1e-9):
        return str(int(round(value)))
    return f"{value:.6f}".rstrip("0").rstrip(".")


def generate_temperatures(
    min_temp: float,
    max_temp: float,
    samples: int,
    step: float | None,
) -> list[float]:
    if step is not None:
        if step <= 0.0:
            raise ValueError("--step must be greater than zero")

        temps: list[float] = []
        current = min_temp
        guard = 0
        max_points = 100000
        while current < max_temp and guard < max_points:
            temps.append(current)
            current += step
            guard += 1

        if guard >= max_points:
            raise ValueError("--step generated too many points")

        if not temps or not math.isclose(temps[-1], max_temp, abs_tol=1e-9):
            temps.append(max_temp)
        return temps

    if samples < 2:
        raise ValueError("--samples must be at least 2")

    spacing = (max_temp - min_temp) / float(samples - 1)
    return [min_temp + (spacing * index) for index in range(samples)]


def build_points(model: ThermistorModel, temperatures: Iterable[float]) -> list[TablePoint]:
    return [
        TablePoint(
            temp_c=temp_c,
            ntc_ohms=model.ntc_resistance(temp_c),
            table_ohms=model.table_resistance(temp_c),
            adc_fraction=model.adc_fraction(temp_c),
        )
        for temp_c in temperatures
    ]


def estimate_interpolation_error(
    model: ThermistorModel,
    points: Sequence[TablePoint],
    min_temp: float,
    max_temp: float,
    dense_points: int,
) -> InterpolationReport:
    """Estimate Kalico's temperature error from linear table interpolation."""
    if dense_points < 2:
        raise ValueError("--error-points must be at least 2")

    # Kalico's LinearResistance helper sorts by resistance, then interpolates the
    # temperature for the measured resistance. The resistance decreases as hotend
    # temperature increases, so sorting puts the hottest point first.
    by_resistance = sorted((point.table_ohms, point.temp_c) for point in points)

    max_error = -1.0
    max_at_temp = min_temp
    max_estimated_temp = min_temp
    total_error = 0.0

    for index in range(dense_points):
        true_temp = min_temp + ((max_temp - min_temp) * index / float(dense_points - 1))
        true_resistance = model.table_resistance(true_temp)
        estimated_temp = interpolate_temperature(by_resistance, true_resistance)
        error = abs(estimated_temp - true_temp)
        total_error += error

        if error > max_error:
            max_error = error
            max_at_temp = true_temp
            max_estimated_temp = estimated_temp

    return InterpolationReport(
        max_error_c=max_error,
        at_temp_c=max_at_temp,
        estimated_temp_c=max_estimated_temp,
        average_error_c=total_error / float(dense_points),
        dense_points=dense_points,
    )


def interpolate_temperature(resistance_points: Sequence[tuple[float, float]], resistance: float) -> float:
    """Linearly interpolate temperature from sorted (resistance, temperature) points."""
    first_resistance, first_temp = resistance_points[0]
    last_resistance, last_temp = resistance_points[-1]

    if resistance <= first_resistance:
        return first_temp
    if resistance >= last_resistance:
        return last_temp

    for (low_resistance, low_temp), (high_resistance, high_temp) in zip(
        resistance_points,
        resistance_points[1:],
    ):
        if low_resistance <= resistance <= high_resistance:
            fraction = (resistance - low_resistance) / (high_resistance - low_resistance)
            return low_temp + (fraction * (high_temp - low_temp))

    raise ValueError("resistance was not covered by the generated table")


def shell_quote_argv(argv: Sequence[str]) -> str:
    return " ".join(shlex.quote(value) for value in argv)


def wrapped_comment_lines(text: str, indent: str = "#   ", width: int = 96) -> list[str]:
    wrapped = textwrap.wrap(
        text,
        width=width - len(indent),
        break_long_words=False,
        break_on_hyphens=False,
    )
    if not wrapped:
        return [indent.rstrip()]
    return [f"{indent}{line}" for line in wrapped]


def format_argument_value(value: object) -> str:
    if value is None:
        return "unset"
    if isinstance(value, float):
        return format_float(value)
    return str(value)


def resolved_argument_lines(args: argparse.Namespace) -> list[str]:
    values = [
        ("output", args.output),
        ("sensor-name", args.sensor_name),
        ("r25", args.r25),
        ("beta", args.beta),
        ("reference-temp", args.reference_temp),
        ("parallel-resistor", args.parallel_resistor),
        ("pullup-resistor", args.pullup_resistor),
        ("min-temp", args.min_temp),
        ("max-temp", args.max_temp),
        ("samples", args.samples if args.step is None else None),
        ("step", args.step),
        ("precision", args.precision),
        ("error-points", args.error_points),
    ]
    return [f"#   {name}: {format_argument_value(value)}" for name, value in values]


def render_config(
    sensor_name: str,
    model: ThermistorModel,
    points: Sequence[TablePoint],
    interpolation_report: InterpolationReport,
    precision: int,
    invocation: str,
    resolved_args: Sequence[str],
) -> str:
    lines = [
        "# Kalico compensated temperature table for the stock Elegoo 100K beta 4300",
        "# thermistor with a fixed resistor in parallel.",
        "#",
        "# Generated by thermistor-tablegen.py.",
        "# Command line:",
        *wrapped_comment_lines(invocation),
        "# Resolved arguments:",
        *resolved_args,
        "#",
        "# Kalico/Danger-Klipper [adc_temperature] docs:",
        f"#   {KALICO_ADC_TEMPERATURE_DOC_URL}",
        "#",
        "# This table is intended for the board topology where the stock NTC",
        "# thermistor is shunted by a fixed resistor. Kalico's thermistor",
        "# inline_resistor option models a series-style correction, so it is not",
        "# used here. Instead, this [adc_temperature] table stores the effective",
        "# resistance that the MCU sees after the parallel resistor is applied.",
        "#",
        "# Math used for every table row:",
        "#   topology: Vref -- pullup -- ADC node -- (NTC || parallel resistor) -- GND",
        "#   T_K = T_C + 273.15",
        "#   R_ntc = R0 * exp(beta * (1 / T_K - 1 / T0_K))",
        "#   R_table = (R_ntc * R_parallel) / (R_ntc + R_parallel)",
        "#   adc_fraction = R_table / (pullup_resistor + R_table)",
        "#",
        f"# R0 at {format_float(model.reference_temp)} C: {format_float(model.r25, 1)} ohms",
        f"# beta: {format_float(model.beta)}",
        f"# parallel resistor: {format_float(model.parallel_resistor, 1)} ohms",
        f"# suggested pullup_resistor setting: {format_float(model.pullup_resistor, 1)} ohms",
        "# No ADC voltage is needed for this resistance table. Kalico works from",
        "# the normalized ADC fraction and pullup_resistor, so the reference",
        "# voltage cancels out.",
        f"# table points: {len(points)}",
        (
            "# estimated worst-case linear interpolation error: "
            f"{interpolation_report.max_error_c:.4f} C near "
            f"{format_float(interpolation_report.at_temp_c)} C"
        ),
        "#",
        "# Include or paste this section above the first heater that uses it, then set",
        f"# the heater's sensor_type to {sensor_name}.",
        "#",
        "# Example [extruder] settings:",
        f"# sensor_type: {sensor_name}",
        f"# pullup_resistor: {format_float(model.pullup_resistor, 1)}",
        "",
        f"[adc_temperature {sensor_name}]",
    ]

    for index, point in enumerate(points, start=1):
        lines.append(f"temperature{index}: {format_float(point.temp_c)}")
        lines.append(f"resistance{index}: {format_float(point.table_ohms, precision)}")

    lines.append("")
    return "\n".join(lines)


def print_verbose_report(
    output: Path,
    sensor_name: str,
    model: ThermistorModel,
    points: Sequence[TablePoint],
    interpolation_report: InterpolationReport,
    used_step: float | None,
) -> None:
    print("Elegoo/Kalico thermistor table generator")
    print("========================================")
    print(f"Output file: {output}")
    print(f"Sensor name: {sensor_name}")
    print("")
    print("Input model:")
    print(f"  R0 / r25:              {model.r25:.6g} ohms")
    print(f"  beta:                  {model.beta:.6g}")
    print(f"  reference temp:        {model.reference_temp:.6g} C")
    print(f"  parallel resistor:     {model.parallel_resistor:.6g} ohms to ground")
    print(f"  pullup resistor note:  {model.pullup_resistor:.6g} ohms")
    print("")
    print("Math:")
    print("  topology: Vref -- pullup -- ADC node -- (NTC || parallel resistor) -- GND")
    print("  T_K = T_C + 273.15")
    print("  R_ntc = R0 * exp(beta * (1 / T_K - 1 / T0_K))")
    print("  R_table = (R_ntc * R_parallel) / (R_ntc + R_parallel)")
    print("  adc_fraction = R_table / (pullup_resistor + R_table)")
    print("  Note: no ADC voltage input is needed for resistance tables.")
    print("  Kalico uses the normalized ADC fraction with pullup_resistor, so Vref cancels out.")
    print("")
    print("Table choices:")
    print(f"  temperature range:     {points[0].temp_c:g} C to {points[-1].temp_c:g} C")
    print(f"  table points:          {len(points)}")
    if used_step is None:
        spacing = (points[-1].temp_c - points[0].temp_c) / float(len(points) - 1)
        print(f"  generated spacing:     {spacing:g} C")
    else:
        print(f"  requested step:        {used_step:g} C")
        if not math.isclose(points[-2].temp_c + used_step, points[-1].temp_c, abs_tol=1e-9):
            print("  final point:           max temperature appended to close the range")
    print("")
    print("Sample rows:")
    for point in representative_points(points):
        print(
            "  "
            f"T={point.temp_c:8.3f} C  "
            f"R_ntc={point.ntc_ohms:12.3f} ohm  "
            f"R_table={point.table_ohms:12.3f} ohm  "
            f"adc_fraction={point.adc_fraction:.6f}"
        )
    print("")
    print("Interpolation estimate:")
    print(
        "  worst-case error:      "
        f"{interpolation_report.max_error_c:.6f} C "
        f"near {interpolation_report.at_temp_c:.3f} C "
        f"(estimated {interpolation_report.estimated_temp_c:.3f} C)"
    )
    print(f"  average abs error:     {interpolation_report.average_error_c:.6f} C")
    print(f"  dense check points:    {interpolation_report.dense_points}")
    print("")
    print("Wrote config successfully.")


def representative_points(points: Sequence[TablePoint]) -> list[TablePoint]:
    indexes = sorted({0, len(points) // 2, len(points) - 1})
    return [points[index] for index in indexes]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        formatter_class=VerboseHelpFormatter,
        description="""\
Generate printer.cfg-elegoon: a Kalico [adc_temperature] resistance table for
the stock Elegoo 100K beta 4300 hotend thermistor when the thermistor signal is
also shunted to ground by a fixed parallel resistor.

Hardware model:
  Vref/ADC supply -> pullup resistor -> ADC node -> (NTC thermistor || fixed
  parallel resistor) -> ground

This emits temperatureN/resistanceN pairs, not voltageN pairs. In that mode,
Kalico uses the heater section's pullup_resistor setting to convert the MCU ADC
ratio back to resistance, then linearly interpolates temperature from this table.
""",
        epilog=f"""\
Approach and parameter meanings:
  --r25
      Bare NTC resistance at --reference-temp. The default stock Elegoo model is
      100000 ohms at 25 C.

  --beta
      Beta value for the bare NTC thermistor. The default stock Elegoo model
      used here is beta 4300.

  --parallel-resistor
      Fixed resistor from the ADC node / thermistor signal to ground. This is
      electrically in parallel with the NTC thermistor. The default is 100000
      ohms, matching the suspected Elegoo board shunt.

  --pullup-resistor
      Fixed resistor from Vref/ADC supply to the ADC node. Kalico's thermistor
      and adc_temperature resistance paths default this to 4700 ohms. This script
      includes it in comments and in the verbose ADC math; the generated table
      itself remains resistance-vs-temperature.

  ADC voltage
      There is intentionally no ADC voltage option for this resistance-table
      generator. Kalico's MCU path reports a normalized ADC fraction for this
      thermistor divider. With temperatureN/resistanceN entries, Kalico combines
      that fraction with pullup_resistor to recover resistance, so the absolute
      ADC/reference voltage is not part of the generated table.

  --samples / --step / --min-temp / --max-temp
      Control how densely the table samples the nonlinear thermistor curve.
      More samples generally reduce Kalico's linear interpolation error. The
      default 33 samples over 0..320 C reproduces a 10 C spacing. Use, for
      example, --step 5 or --samples 65 for a denser table.

  --precision
      Number of decimal places written for resistanceN values. One decimal is
      already much finer than real resistor/thermistor tolerance, but keeping it
      explicit makes generated diffs stable.

Docs:
  {KALICO_ADC_TEMPERATURE_DOC_URL}

Examples:
  ./thermistor-tablegen.py
  ./thermistor-tablegen.py -o printer.cfg-elegoon --step 5
  ./thermistor-tablegen.py --r25 100000 --beta 4300 --parallel-resistor 100000
""",
    )
    parser.add_argument(
        "-o",
        "--output",
        default=DEFAULT_OUTPUT,
        help="output config filename",
    )
    parser.add_argument(
        "--sensor-name",
        default=DEFAULT_SENSOR_NAME,
        help="name used in [adc_temperature <name>] and sensor_type",
    )
    parser.add_argument(
        "--r25",
        type=float,
        default=100000.0,
        help="bare thermistor resistance at --reference-temp, in ohms",
    )
    parser.add_argument(
        "--beta",
        type=float,
        default=4300.0,
        help="bare thermistor beta value used by the NTC beta equation",
    )
    parser.add_argument(
        "--reference-temp",
        type=float,
        default=25.0,
        help="Celsius temperature where --r25 applies",
    )
    parser.add_argument(
        "--parallel-resistor",
        type=float,
        default=100000.0,
        help="fixed resistor from ADC node / thermistor signal to ground, in ohms",
    )
    parser.add_argument(
        "--pullup-resistor",
        type=float,
        default=4700.0,
        help="resistor from Vref/ADC supply to ADC node, in ohms",
    )
    parser.add_argument("--min-temp", type=float, default=0.0, help="minimum table temperature in Celsius")
    parser.add_argument("--max-temp", type=float, default=320.0, help="maximum table temperature in Celsius")

    sample_group = parser.add_mutually_exclusive_group()
    sample_group.add_argument(
        "--samples",
        type=int,
        default=33,
        help="number of evenly spaced table samples, including endpoints",
    )
    sample_group.add_argument(
        "--step",
        type=float,
        help="temperature step in Celsius; appends max-temp if the step does not land exactly",
    )

    parser.add_argument(
        "--precision",
        type=int,
        default=1,
        help="decimal places for generated resistance values",
    )
    parser.add_argument(
        "--error-points",
        type=int,
        default=5000,
        help="dense points used to estimate interpolation error",
    )
    return parser.parse_args()


def validate_args(args: argparse.Namespace) -> None:
    positive_fields = (
        ("--r25", args.r25),
        ("--beta", args.beta),
        ("--parallel-resistor", args.parallel_resistor),
        ("--pullup-resistor", args.pullup_resistor),
    )
    for name, value in positive_fields:
        if value <= 0.0:
            raise ValueError(f"{name} must be greater than zero")

    if args.max_temp <= args.min_temp:
        raise ValueError("--max-temp must be greater than --min-temp")
    if args.precision < 0:
        raise ValueError("--precision must be zero or greater")
    if args.error_points < 2:
        raise ValueError("--error-points must be at least 2")

    celsius_to_kelvin(args.min_temp)
    celsius_to_kelvin(args.max_temp)
    celsius_to_kelvin(args.reference_temp)


def main() -> int:
    args = parse_args()

    try:
        validate_args(args)
        model = ThermistorModel(
            r25=args.r25,
            beta=args.beta,
            reference_temp=args.reference_temp,
            parallel_resistor=args.parallel_resistor,
            pullup_resistor=args.pullup_resistor,
        )
        temperatures = generate_temperatures(
            min_temp=args.min_temp,
            max_temp=args.max_temp,
            samples=args.samples,
            step=args.step,
        )
        points = build_points(model, temperatures)
        interpolation_report = estimate_interpolation_error(
            model=model,
            points=points,
            min_temp=args.min_temp,
            max_temp=args.max_temp,
            dense_points=args.error_points,
        )
        config = render_config(
            sensor_name=args.sensor_name,
            model=model,
            points=points,
            interpolation_report=interpolation_report,
            precision=args.precision,
            invocation=shell_quote_argv(sys.argv),
            resolved_args=resolved_argument_lines(args),
        )

        output = Path(args.output)
        output.write_text(config, encoding="utf-8")
        print_verbose_report(
            output=output,
            sensor_name=args.sensor_name,
            model=model,
            points=points,
            interpolation_report=interpolation_report,
            used_step=args.step,
        )
        return 0
    except ValueError as exc:
        print(f"error: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
