#!/usr/bin/env python3
"""
Replay an adc_temperature table through the stock Elegoo 1.4.44 thermistor math.

This is a diagnostic tool for comparing:

  1. The generated Kalico [adc_temperature] resistance table.
  2. The source-equivalent / decompiled Elegoo 1.4.44 thermistor code.
  3. Real measurements where Kalico reported one temperature and an external
     thermometer reported another.

Important naming detail:
  Elegoo's 1.4.44 config calls the extra resistor "inline_resistor", but the
  source-equivalent code treats it as a resistor in parallel with the thermistor:

      r_thermistor = (r_total * inline_resistor) / (inline_resistor - r_total)
      r_total = (r_thermistor * inline_resistor) / (r_thermistor + inline_resistor)

  This script intentionally keeps the "inline_resistor" option name because that
  is what the 1.4.44 firmware used, but the math below is parallel-resistor math.

Stock code mirrored here:
  /home/paul/carbon/cc-firmware/core/klippy/extras/thermistor.cpp

Relevant functions:
  Thermistor::setup_coefficients_beta(...)
  Thermistor::calc_temp(...)
  Thermistor::calc_adc(...)
"""

from __future__ import annotations

import argparse
import math
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Sequence


KELVIN_TO_CELSIUS = -273.15
INLINE_RESISTOR_OFFSET = 3000.0
EPSILON = 1e-15

DEFAULT_TABLE = Path(__file__).resolve().with_name("printer.cfg-elegoon")
DEFAULT_OBSERVED = "50:44,100:90,150:136,200:180,250:219,300:260"


@dataclass(frozen=True)
class TableEntry:
    index: int
    temp_c: float
    resistance_ohms: float


@dataclass(frozen=True)
class ObservedPair:
    kalico_c: float
    external_c: float


@dataclass(frozen=True)
class ObservedResistance:
    kalico_c: float
    external_c: float
    measured_resistance_ohms: float


@dataclass(frozen=True)
class FitResult:
    name: str
    r25: float
    beta: float
    parallel_resistor: float
    rms_temp_error_c: float
    max_abs_temp_error_c: float
    rms_log_resistance_error: float


class StockFirmwareThermistor:
    """Python equivalent of the stock Elegoo 1.4.44 beta thermistor path."""

    def __init__(
        self,
        pullup_resistor: float,
        inline_resistor: float,
        reference_temp: float,
        r25: float,
        beta: float,
    ) -> None:
        self.pullup_resistor = pullup_resistor
        self.inline_resistor = inline_resistor
        self.reference_temp = reference_temp
        self.r25 = r25
        self.beta = beta
        self.c1 = 0.0
        self.c2 = 0.0
        self.c3 = 0.0
        self.setup_coefficients_beta(reference_temp, r25, beta)

    def setup_coefficients_beta(self, t1: float, r1: float, beta: float) -> None:
        """Mirror Thermistor::setup_coefficients_beta(t1, r1, beta)."""
        inv_t1 = 1.0 / (t1 - KELVIN_TO_CELSIUS)
        ln_r1 = math.log(r1)
        self.c3 = 0.0
        self.c2 = 1.0 / beta
        self.c1 = inv_t1 - (self.c2 * ln_r1)

    def calc_temp_from_adc(self, adc: float) -> float:
        """Mirror Thermistor::calc_temp(..., adc), minus shutdown side effects."""
        adc = max(0.00001, min(0.99999, adc))
        measured_resistance = self.pullup_resistor * adc / (1.0 - adc)
        thermistor_resistance = self.bare_resistance_from_measured(measured_resistance)

        ln_r = math.log(thermistor_resistance)
        inv_t = self.c1 + (self.c2 * ln_r) + (self.c3 * pow(ln_r, 3))
        return (1.0 / inv_t) + KELVIN_TO_CELSIUS

    def calc_adc(self, temp_c: float) -> float:
        """Mirror Thermistor::calc_adc(temp)."""
        if temp_c <= KELVIN_TO_CELSIUS:
            return 1.0

        thermistor_resistance = self.bare_resistance_at_temp(temp_c)
        measured_resistance = self.measured_resistance_from_bare(thermistor_resistance)
        return measured_resistance / (self.pullup_resistor + measured_resistance)

    def calc_temp_from_measured_resistance(self, measured_resistance: float) -> float:
        adc = measured_resistance / (self.pullup_resistor + measured_resistance)
        return self.calc_temp_from_adc(adc)

    def bare_resistance_at_temp(self, temp_c: float) -> float:
        inv_t = 1.0 / (temp_c - KELVIN_TO_CELSIUS)
        if abs(self.c3) > EPSILON:
            y = (self.c1 - inv_t) / (2.0 * self.c3)
            x = math.sqrt(pow((self.c2 / (3.0 * self.c3)), 3) + pow(y, 2))
            ln_r = signed_cuberoot(x - y) - signed_cuberoot(x + y)
        else:
            ln_r = (inv_t - self.c1) / self.c2
        return math.exp(ln_r)

    def measured_resistance_at_temp(self, temp_c: float) -> float:
        return self.measured_resistance_from_bare(self.bare_resistance_at_temp(temp_c))

    def measured_resistance_from_bare(self, thermistor_resistance: float) -> float:
        if abs(self.inline_resistor) > EPSILON:
            return (
                thermistor_resistance
                * self.inline_resistor
                / (thermistor_resistance + self.inline_resistor)
            )
        return thermistor_resistance

    def bare_resistance_from_measured(self, measured_resistance: float) -> float:
        if abs(self.inline_resistor) > EPSILON:
            if self.inline_resistor - measured_resistance > EPSILON:
                return (
                    measured_resistance
                    * self.inline_resistor
                    / (self.inline_resistor - measured_resistance)
                )
            return self.inline_resistor
        return measured_resistance

    def would_trip_inline_guard(self, measured_resistance: float) -> bool:
        if abs(self.inline_resistor) <= EPSILON:
            return False
        return measured_resistance > self.inline_resistor - INLINE_RESISTOR_OFFSET


def signed_cuberoot(value: float) -> float:
    if value < 0.0:
        return -pow(-value, 1.0 / 3.0)
    return pow(value, 1.0 / 3.0)


def parse_table(path: Path) -> list[TableEntry]:
    temperatures: dict[int, float] = {}
    resistances: dict[int, float] = {}
    temp_pattern = re.compile(r"^temperature(\d+)\s*:\s*([-+0-9.eE]+)\s*$")
    resistance_pattern = re.compile(r"^resistance(\d+)\s*:\s*([-+0-9.eE]+)\s*$")

    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue

        temp_match = temp_pattern.match(line)
        if temp_match:
            temperatures[int(temp_match.group(1))] = float(temp_match.group(2))
            continue

        resistance_match = resistance_pattern.match(line)
        if resistance_match:
            resistances[int(resistance_match.group(1))] = float(resistance_match.group(2))
            continue

        if line.startswith("temperature") or line.startswith("resistance"):
            raise ValueError(f"{path}:{line_number}: could not parse table line: {line}")

    indexes = sorted(set(temperatures) | set(resistances))
    if not indexes:
        raise ValueError(f"{path}: no temperatureN/resistanceN entries found")

    missing = [index for index in indexes if index not in temperatures or index not in resistances]
    if missing:
        raise ValueError(f"{path}: missing temperature or resistance for indexes: {missing}")

    return [
        TableEntry(index=index, temp_c=temperatures[index], resistance_ohms=resistances[index])
        for index in indexes
    ]


def parse_observed_pairs(raw: str) -> list[ObservedPair]:
    if not raw.strip():
        return []

    pairs: list[ObservedPair] = []
    for item in raw.split(","):
        item = item.strip()
        if not item:
            continue
        if ":" not in item:
            raise ValueError(f"observed pair must be Kalico:External, got: {item}")
        kalico, external = item.split(":", 1)
        pairs.append(ObservedPair(kalico_c=float(kalico), external_c=float(external)))
    return pairs


def interpolate_resistance_for_temp(entries: Sequence[TableEntry], temp_c: float) -> float:
    ordered = sorted(entries, key=lambda entry: entry.temp_c)
    if temp_c < ordered[0].temp_c or temp_c > ordered[-1].temp_c:
        raise ValueError(
            f"{temp_c:g} C is outside table range {ordered[0].temp_c:g}..{ordered[-1].temp_c:g} C"
        )

    for entry in ordered:
        if math.isclose(entry.temp_c, temp_c, abs_tol=1e-12):
            return entry.resistance_ohms

    for low, high in zip(ordered, ordered[1:]):
        if low.temp_c <= temp_c <= high.temp_c:
            fraction = (temp_c - low.temp_c) / (high.temp_c - low.temp_c)
            return low.resistance_ohms + (fraction * (high.resistance_ohms - low.resistance_ohms))

    raise ValueError(f"could not interpolate resistance for {temp_c:g} C")


def implied_beta(
    thermistor: StockFirmwareThermistor,
    measured_resistance: float,
    actual_temp_c: float,
) -> float | None:
    bare_resistance = thermistor.bare_resistance_from_measured(measured_resistance)
    denominator = (
        (1.0 / (actual_temp_c - KELVIN_TO_CELSIUS))
        - (1.0 / (thermistor.reference_temp - KELVIN_TO_CELSIUS))
    )
    if abs(denominator) <= EPSILON or bare_resistance <= 0.0:
        return None
    return math.log(bare_resistance / thermistor.r25) / denominator


def thermistor_resistance_at_temp(
    temp_c: float,
    reference_temp: float,
    r25: float,
    beta: float,
) -> float:
    temp_k = temp_c - KELVIN_TO_CELSIUS
    reference_k = reference_temp - KELVIN_TO_CELSIUS
    return r25 * math.exp(beta * ((1.0 / temp_k) - (1.0 / reference_k)))


def parallel_resistance(thermistor_resistance: float, parallel_ohms: float) -> float:
    return (
        thermistor_resistance
        * parallel_ohms
        / (thermistor_resistance + parallel_ohms)
    )


def measured_resistance_for_params(
    temp_c: float,
    reference_temp: float,
    r25: float,
    beta: float,
    parallel_ohms: float,
) -> float:
    return parallel_resistance(
        thermistor_resistance_at_temp(temp_c, reference_temp, r25, beta),
        parallel_ohms,
    )


def temp_from_measured_resistance_for_params(
    measured_resistance: float,
    reference_temp: float,
    r25: float,
    beta: float,
    parallel_ohms: float,
) -> float:
    if parallel_ohms - measured_resistance <= EPSILON:
        return float("nan")
    bare_resistance = measured_resistance * parallel_ohms / (parallel_ohms - measured_resistance)
    inv_t = (1.0 / (reference_temp - KELVIN_TO_CELSIUS)) + (
        math.log(bare_resistance / r25) / beta
    )
    return (1.0 / inv_t) + KELVIN_TO_CELSIUS


def observed_resistances_from_pairs(
    entries: Sequence[TableEntry],
    observations: Sequence[ObservedPair],
) -> list[ObservedResistance]:
    return [
        ObservedResistance(
            kalico_c=observed.kalico_c,
            external_c=observed.external_c,
            measured_resistance_ohms=interpolate_resistance_for_temp(entries, observed.kalico_c),
        )
        for observed in observations
    ]


def fit_quality(
    name: str,
    observations: Sequence[ObservedResistance],
    reference_temp: float,
    r25: float,
    beta: float,
    parallel_ohms: float,
) -> FitResult:
    temp_errors: list[float] = []
    log_errors: list[float] = []

    for observed in observations:
        predicted_temp = temp_from_measured_resistance_for_params(
            measured_resistance=observed.measured_resistance_ohms,
            reference_temp=reference_temp,
            r25=r25,
            beta=beta,
            parallel_ohms=parallel_ohms,
        )
        temp_errors.append(predicted_temp - observed.external_c)

        predicted_resistance = measured_resistance_for_params(
            temp_c=observed.external_c,
            reference_temp=reference_temp,
            r25=r25,
            beta=beta,
            parallel_ohms=parallel_ohms,
        )
        log_errors.append(math.log(predicted_resistance / observed.measured_resistance_ohms))

    rms_temp_error = math.sqrt(sum(error * error for error in temp_errors) / len(temp_errors))
    max_abs_temp_error = max(abs(error) for error in temp_errors)
    rms_log_resistance_error = math.sqrt(sum(error * error for error in log_errors) / len(log_errors))
    return FitResult(
        name=name,
        r25=r25,
        beta=beta,
        parallel_resistor=parallel_ohms,
        rms_temp_error_c=rms_temp_error,
        max_abs_temp_error_c=max_abs_temp_error,
        rms_log_resistance_error=rms_log_resistance_error,
    )


def solve_r25_beta_for_parallel(
    observations: Sequence[ObservedResistance],
    reference_temp: float,
    parallel_ohms: float,
    fixed_r25: float | None,
    fixed_beta: float | None,
) -> tuple[float, float] | None:
    max_measured = max(observed.measured_resistance_ohms for observed in observations)
    if parallel_ohms <= max_measured + EPSILON:
        return None

    xs: list[float] = []
    ys: list[float] = []
    reference_k = reference_temp - KELVIN_TO_CELSIUS
    for observed in observations:
        bare_resistance = (
            observed.measured_resistance_ohms
            * parallel_ohms
            / (parallel_ohms - observed.measured_resistance_ohms)
        )
        xs.append((1.0 / (observed.external_c - KELVIN_TO_CELSIUS)) - (1.0 / reference_k))
        ys.append(math.log(bare_resistance))

    if fixed_r25 is not None and fixed_beta is not None:
        return fixed_r25, fixed_beta

    if fixed_r25 is not None:
        intercept = math.log(fixed_r25)
        denominator = sum(x * x for x in xs)
        if denominator <= EPSILON:
            return None
        beta = sum(x * (y - intercept) for x, y in zip(xs, ys)) / denominator
        return fixed_r25, beta

    if fixed_beta is not None:
        intercept = sum(y - (fixed_beta * x) for x, y in zip(xs, ys)) / len(xs)
        return math.exp(intercept), fixed_beta

    x_mean = sum(xs) / len(xs)
    y_mean = sum(ys) / len(ys)
    denominator = sum((x - x_mean) * (x - x_mean) for x in xs)
    if denominator <= EPSILON:
        return None
    beta = sum((x - x_mean) * (y - y_mean) for x, y in zip(xs, ys)) / denominator
    intercept = y_mean - (beta * x_mean)
    return math.exp(intercept), beta


def golden_section_search(
    objective: Callable[[float], float],
    low: float,
    high: float,
    iterations: int = 140,
) -> float:
    golden = (math.sqrt(5.0) - 1.0) / 2.0
    x1 = high - golden * (high - low)
    x2 = low + golden * (high - low)
    y1 = objective(x1)
    y2 = objective(x2)

    for _ in range(iterations):
        if y1 > y2:
            low = x1
            x1 = x2
            y1 = y2
            x2 = low + golden * (high - low)
            y2 = objective(x2)
        else:
            high = x2
            x2 = x1
            y2 = y1
            x1 = high - golden * (high - low)
            y1 = objective(x1)

    return (low + high) / 2.0


def fit_parallel_search(
    name: str,
    observations: Sequence[ObservedResistance],
    reference_temp: float,
    fixed_r25: float | None,
    fixed_beta: float | None,
    min_parallel: float,
    max_parallel: float,
) -> FitResult:
    min_log = math.log(min_parallel)
    max_log = math.log(max_parallel)

    def objective(log_parallel: float) -> float:
        parallel_ohms = math.exp(log_parallel)
        solved = solve_r25_beta_for_parallel(
            observations,
            reference_temp,
            parallel_ohms,
            fixed_r25=fixed_r25,
            fixed_beta=fixed_beta,
        )
        if solved is None:
            return float("inf")
        r25, beta = solved
        return fit_quality(
            name,
            observations,
            reference_temp,
            r25,
            beta,
            parallel_ohms,
        ).rms_log_resistance_error

    best_log_parallel = golden_section_search(objective, min_log, max_log)
    best_parallel = math.exp(best_log_parallel)
    solved = solve_r25_beta_for_parallel(
        observations,
        reference_temp,
        best_parallel,
        fixed_r25=fixed_r25,
        fixed_beta=fixed_beta,
    )
    if solved is None:
        raise ValueError("could not solve fitted thermistor parameters")
    r25, beta = solved
    return fit_quality(name, observations, reference_temp, r25, beta, best_parallel)


def fit_observed_models(
    observations: Sequence[ObservedResistance],
    reference_temp: float,
    r25: float,
    beta: float,
    parallel_ohms: float,
) -> list[FitResult]:
    if not observations:
        return []

    max_measured = max(observed.measured_resistance_ohms for observed in observations)
    min_parallel = max(max_measured * 1.001, 1000.0)
    max_parallel = 10_000_000.0

    results = [
        fit_quality(
            "current inputs",
            observations,
            reference_temp,
            r25,
            beta,
            parallel_ohms,
        )
    ]

    fixed_parallel_fit = solve_r25_beta_for_parallel(
        observations,
        reference_temp,
        parallel_ohms,
        fixed_r25=r25,
        fixed_beta=None,
    )
    if fixed_parallel_fit is not None:
        fitted_r25, fitted_beta = fixed_parallel_fit
        results.append(
            fit_quality(
                "fit beta only; keep R25 and parallel",
                observations,
                reference_temp,
                fitted_r25,
                fitted_beta,
                parallel_ohms,
            )
        )

    fixed_parallel_free_r25_fit = solve_r25_beta_for_parallel(
        observations,
        reference_temp,
        parallel_ohms,
        fixed_r25=None,
        fixed_beta=None,
    )
    if fixed_parallel_free_r25_fit is not None:
        fitted_r25, fitted_beta = fixed_parallel_free_r25_fit
        results.append(
            fit_quality(
                "fit R25 and beta; keep parallel",
                observations,
                reference_temp,
                fitted_r25,
                fitted_beta,
                parallel_ohms,
            )
        )

    results.append(
        fit_parallel_search(
            "fit parallel only; keep R25 and beta",
            observations,
            reference_temp,
            fixed_r25=r25,
            fixed_beta=beta,
            min_parallel=min_parallel,
            max_parallel=max_parallel,
        )
    )
    results.append(
        fit_parallel_search(
            "fit beta and parallel; keep R25",
            observations,
            reference_temp,
            fixed_r25=r25,
            fixed_beta=None,
            min_parallel=min_parallel,
            max_parallel=max_parallel,
        )
    )
    results.append(
        fit_parallel_search(
            "fit R25, beta, and parallel",
            observations,
            reference_temp,
            fixed_r25=None,
            fixed_beta=None,
            min_parallel=min_parallel,
            max_parallel=max_parallel,
        )
    )
    return sorted(results, key=lambda result: result.rms_temp_error_c)


def print_fit_report(
    entries: Sequence[TableEntry],
    thermistor: StockFirmwareThermistor,
    observations: Sequence[ObservedPair],
) -> None:
    if not observations:
        return

    observed_resistances = observed_resistances_from_pairs(entries, observations)
    fits = fit_observed_models(
        observed_resistances,
        reference_temp=thermistor.reference_temp,
        r25=thermistor.r25,
        beta=thermistor.beta,
        parallel_ohms=thermistor.inline_resistor,
    )

    print("Parameter fits from observed measurements")
    print("=========================================")
    print(
        " model                                  "
        "R25_ohm      beta   parallel_ohm   rms_temp_C  max_temp_C  rms_lnR"
    )
    for result in fits:
        print(
            f"{result.name[:37]:37s}"
            f"  {result.r25:9.1f}"
            f"  {result.beta:8.1f}"
            f"  {result.parallel_resistor:13.1f}"
            f"  {result.rms_temp_error_c:10.3f}"
            f"  {result.max_abs_temp_error_c:10.3f}"
            f"  {result.rms_log_resistance_error:7.4f}"
        )

    print("")
    print("Best-fit row residuals")
    print("======================")
    best = fits[0]
    print(
        f"Using: R25={best.r25:.1f} ohms, beta={best.beta:.1f}, "
        f"parallel={best.parallel_resistor:.1f} ohms"
    )
    print(" Kalico_C  External_C  R_inferred  fitted_temp_C  residual_C")
    for observed in observed_resistances:
        fitted_temp = temp_from_measured_resistance_for_params(
            measured_resistance=observed.measured_resistance_ohms,
            reference_temp=thermistor.reference_temp,
            r25=best.r25,
            beta=best.beta,
            parallel_ohms=best.parallel_resistor,
        )
        print(
            f"{observed.kalico_c:9.3f}"
            f"  {observed.external_c:10.3f}"
            f"  {observed.measured_resistance_ohms:10.3f}"
            f"  {fitted_temp:13.3f}"
            f"  {fitted_temp - observed.external_c:10.3f}"
        )
    print("")


def print_table_self_check(entries: Sequence[TableEntry], thermistor: StockFirmwareThermistor) -> float:
    print("Table self-check through stock 1.4.44 math")
    print("==========================================")
    print(
        " idx  table_C    table_R_ohm   adc_fraction   stock_calc_C   delta_C   guard"
    )

    max_abs_delta = 0.0
    for entry in entries:
        adc = entry.resistance_ohms / (thermistor.pullup_resistor + entry.resistance_ohms)
        stock_temp = thermistor.calc_temp_from_adc(adc)
        delta = stock_temp - entry.temp_c
        max_abs_delta = max(max_abs_delta, abs(delta))
        guard = "YES" if thermistor.would_trip_inline_guard(entry.resistance_ohms) else "no"
        print(
            f"{entry.index:4d}"
            f"  {entry.temp_c:8.3f}"
            f"  {entry.resistance_ohms:13.3f}"
            f"  {adc:13.6f}"
            f"  {stock_temp:13.4f}"
            f"  {delta:8.4f}"
            f"  {guard:>5}"
        )

    print("")
    print(f"Max absolute table-vs-stock delta: {max_abs_delta:.6f} C")
    print("")
    return max_abs_delta


def print_observed_comparison(
    entries: Sequence[TableEntry],
    thermistor: StockFirmwareThermistor,
    observations: Sequence[ObservedPair],
) -> None:
    if not observations:
        return

    print("Observed Kalico vs external measurements")
    print("========================================")
    print(
        " Kalico_C  External_C  Error_C  "
        "R_from_Kalico  Stock_C_from_R  R_expected_at_external  R_delta  implied_beta"
    )

    errors: list[float] = []
    implied_betas: list[float] = []

    for observed in observations:
        inferred_resistance = interpolate_resistance_for_temp(entries, observed.kalico_c)
        stock_temp = thermistor.calc_temp_from_measured_resistance(inferred_resistance)
        expected_resistance = thermistor.measured_resistance_at_temp(observed.external_c)
        resistance_delta = inferred_resistance - expected_resistance
        temp_error = observed.kalico_c - observed.external_c
        beta = implied_beta(thermistor, inferred_resistance, observed.external_c)
        errors.append(temp_error)
        if beta is not None:
            implied_betas.append(beta)

        beta_text = "n/a" if beta is None else f"{beta:12.1f}"
        print(
            f"{observed.kalico_c:9.3f}"
            f"  {observed.external_c:10.3f}"
            f"  {temp_error:7.3f}"
            f"  {inferred_resistance:13.3f}"
            f"  {stock_temp:14.4f}"
            f"  {expected_resistance:22.3f}"
            f"  {resistance_delta:8.3f}"
            f"  {beta_text}"
        )

    print("")
    print(
        "Temperature error summary: "
        f"min {min(errors):.3f} C, max {max(errors):.3f} C, "
        f"average {sum(errors) / len(errors):.3f} C"
    )
    if implied_betas:
        print(
            "Implied beta summary from external readings: "
            f"min {min(implied_betas):.1f}, max {max(implied_betas):.1f}, "
            f"average {sum(implied_betas) / len(implied_betas):.1f}"
        )
    print("")


def print_interpretation(max_abs_delta: float, observations: Sequence[ObservedPair]) -> None:
    print("Interpretation")
    print("==============")
    if max_abs_delta < 0.05:
        print(
            "The generated table is internally consistent with the stock 1.4.44 "
            "parallel-inline thermistor math. Small deltas are from resistance "
            "rounding in printer.cfg."
        )
    else:
        print(
            "The generated table does not exactly replay through the stock 1.4.44 "
            "math. Check the table inputs, rounding, and pullup/inline values."
        )

    if observations:
        print(
            "Your observed data shows Kalico reporting hotter than the external "
            "measurement. In table terms, the inferred measured resistance is "
            "lower than the 100K/B4300 plus 100K-parallel model expects at those "
            "external temperatures. That points to a model or measurement mismatch "
            "rather than the table failing to match the stock firmware formula."
        )
    print("")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Replay printer.cfg-elegoon through the stock Elegoo 1.4.44 "
            "thermistor calculation and compare it with observed readings."
        )
    )
    parser.add_argument(
        "--table",
        type=Path,
        default=DEFAULT_TABLE,
        help=f"adc_temperature table to parse (default: {DEFAULT_TABLE})",
    )
    parser.add_argument("--pullup-resistor", type=float, default=4700.0)
    parser.add_argument(
        "--inline-resistor",
        type=float,
        default=100000.0,
        help="stock firmware config name; stock 1.4.44 uses this as a parallel resistor",
    )
    parser.add_argument("--r25", type=float, default=100000.0)
    parser.add_argument("--beta", type=float, default=4300.0)
    parser.add_argument("--reference-temp", type=float, default=25.0)
    parser.add_argument(
        "--observed",
        default=DEFAULT_OBSERVED,
        help="comma-separated Kalico:External pairs; pass an empty string to disable",
    )
    return parser.parse_args()


def validate_positive(name: str, value: float) -> None:
    if value <= 0.0:
        raise ValueError(f"{name} must be greater than zero")


def main() -> int:
    args = parse_args()

    try:
        validate_positive("--pullup-resistor", args.pullup_resistor)
        validate_positive("--inline-resistor", args.inline_resistor)
        validate_positive("--r25", args.r25)
        validate_positive("--beta", args.beta)

        entries = parse_table(args.table)
        observations = parse_observed_pairs(args.observed)
        thermistor = StockFirmwareThermistor(
            pullup_resistor=args.pullup_resistor,
            inline_resistor=args.inline_resistor,
            reference_temp=args.reference_temp,
            r25=args.r25,
            beta=args.beta,
        )

        print("Stock Elegoo 1.4.44 thermistor check")
        print("====================================")
        print(f"Table:             {args.table}")
        print(f"Rows:              {len(entries)}")
        print(f"Pullup resistor:   {args.pullup_resistor:g} ohms")
        print(
            "Inline resistor:   "
            f"{args.inline_resistor:g} ohms "
            "(stock name; used as parallel resistor in 1.4.44 math)"
        )
        print(f"Thermistor model:  R{args.reference_temp:g}={args.r25:g} ohms, beta={args.beta:g}")
        print("")

        max_abs_delta = print_table_self_check(entries, thermistor)
        print_observed_comparison(entries, thermistor, observations)
        print_fit_report(entries, thermistor, observations)
        print_interpretation(max_abs_delta, observations)
        return 0
    except (OSError, ValueError) as exc:
        print(f"error: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
