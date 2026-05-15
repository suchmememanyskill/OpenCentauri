# Thermistor Table Generator

`thermistor-tablegen.py` generates a Kalico/Klipper `[adc_temperature]`
resistance table for the stock Elegoo hotend thermistor when the board has a
fixed resistor in parallel with the thermistor.

The current defaults target the suspected Centauri Carbon hotend circuit:

- Stock Elegoo NTC thermistor: `100000` ohms at `25 C`
- Beta value: `4300`
- Fixed parallel resistor: `100000` ohms from the ADC node / thermistor signal to ground
- Pullup resistor: `4700` ohms from Vref / ADC supply to the ADC node
- Output file: `printer.cfg-elegoon`
- Sensor name: `elegoo_100k_b4300_parallel_100k`
- Temperature range: `0 C` through `320 C`
- Table density: `33` points, which is `10 C` spacing over the default range

Kalico/Danger-Klipper documentation for this config section:

https://dangerklipper.io/Config_Reference.html#adc_temperature

## Quick Start

Run from this directory:

```bash
./thermistor-tablegen.py
```

Or run from the repository root:

```bash
python3 thermistor-tablegen/thermistor-tablegen.py
```

By default this writes:

```text
printer.cfg-elegoon
```

To write somewhere else:

```bash
./thermistor-tablegen.py -o my-elegoo-thermistor.cfg
```

To generate a denser table with `5 C` spacing:

```bash
./thermistor-tablegen.py --step 5
```

To see the full CLI reference:

```bash
./thermistor-tablegen.py --help
```

## Generated Config Usage

The generated file contains an `[adc_temperature ...]` section. Include or paste
that section above the first heater that uses it, then set the heater sensor to
the generated sensor name.

Example:

```ini
[include printer.cfg-elegoon]

[extruder]
sensor_type: elegoo_100k_b4300_parallel_100k
pullup_resistor: 4700
```

The `pullup_resistor` belongs on the heater section, not in the
`[adc_temperature]` section. Kalico uses it to convert the normalized MCU ADC
reading back into resistance before interpolating temperature from the table.

## Hardware Model

This generator assumes this divider topology:

```text
Vref / ADC supply
    |
    |
 pullup resistor
    |
    +---- ADC input
    |
    +---- NTC thermistor ---- ground
    |
    +---- fixed parallel resistor ---- ground
```

Another way to write the lower leg is:

```text
NTC thermistor || fixed parallel resistor
```

The fixed parallel resistor is the important part. A normal beta thermistor
configuration models only the NTC. If the board also has a fixed shunt resistor
to ground, the MCU sees a lower effective resistance than the bare thermistor,
especially at low temperatures.

## Why `[adc_temperature]`

Kalico supports custom thermistors with `[thermistor ...]`, but that section is
for bare thermistor curves. It does not directly model a fixed resistor in
parallel with the thermistor.

Kalico also exposes `inline_resistor` for common thermistor sensors. That is not
the correct correction for this board topology. `inline_resistor` models an
extra fixed resistor inline with the thermistor path, effectively a series-style
correction. A resistor connected from the ADC node to ground is a parallel
shunt, so it must be folded into the resistance table instead.

`[adc_temperature]` is the right shape because it allows a table of true
temperature to effective measured resistance:

```ini
[adc_temperature my_sensor]
temperature1: ...
resistance1: ...
temperature2: ...
resistance2: ...
```

Kalico then linearly interpolates between those table points.

## Math

The script computes the bare NTC resistance first, then computes the equivalent
parallel resistance that the MCU actually sees.

Convert Celsius to Kelvin:

```text
T_K = T_C + 273.15
T0_K = reference_temp_C + 273.15
```

Compute the bare NTC resistance with the beta equation:

```text
R_ntc(T) = R0 * exp(beta * (1 / T_K - 1 / T0_K))
```

With the defaults:

```text
R0 = 100000 ohms
reference_temp_C = 25 C
beta = 4300
```

Combine the bare NTC with the fixed parallel resistor:

```text
R_table(T) = (R_ntc(T) * R_parallel) / (R_ntc(T) + R_parallel)
```

That `R_table` value is what goes into the generated `resistanceN` entries.

For reference, the divider ratio is:

```text
adc_fraction = R_table / (pullup_resistor + R_table)
```

The script prints representative `adc_fraction` values as a sanity check, but
the generated config is a resistance table, not a voltage table.

## Why ADC Voltage Is Not an Input

This generator intentionally does not have an ADC voltage option.

For resistance-based thermistor tables, Kalico works from the MCU's normalized
ADC fraction. It combines that normalized reading with `pullup_resistor` to
recover the sensor resistance, then interpolates temperature from the
`temperatureN` / `resistanceN` table.

Because the ADC reading is normalized, the absolute ADC reference voltage
cancels out for this use case. Voltage matters for `voltageN` style
`[adc_temperature]` tables and some amplifier-style sensors, but this generator
emits `resistanceN` entries.

## Defaults

| Option | Default | Meaning |
| --- | ---: | --- |
| `-o`, `--output` | `printer.cfg-elegoon` | Config file to write. |
| `--sensor-name` | `elegoo_100k_b4300_parallel_100k` | Name used in `[adc_temperature <name>]` and heater `sensor_type`. |
| `--r25` | `100000.0` | Bare thermistor resistance at `--reference-temp`, in ohms. |
| `--beta` | `4300.0` | Bare thermistor beta value. |
| `--reference-temp` | `25.0` | Celsius temperature where `--r25` applies. |
| `--parallel-resistor` | `100000.0` | Fixed resistor from ADC node / thermistor signal to ground, in ohms. |
| `--pullup-resistor` | `4700.0` | Fixed pullup from Vref / ADC supply to ADC node, in ohms. |
| `--min-temp` | `0.0` | Lowest generated table temperature, in Celsius. |
| `--max-temp` | `320.0` | Highest generated table temperature, in Celsius. |
| `--samples` | `33` | Number of evenly spaced table points, including both endpoints. |
| `--step` | unset | Temperature step in Celsius. Mutually exclusive with `--samples`. |
| `--precision` | `1` | Decimal places written for resistance values. |
| `--error-points` | `5000` | Dense points used to estimate interpolation error. |

## Sampling Density

`[adc_temperature]` uses linear interpolation between table entries. The
thermistor curve is nonlinear, so denser sampling usually reduces interpolation
error.

With the default values and `33` samples over `0..320 C`, the table spacing is
`10 C`. In a local run, the script estimated about `0.42 C` worst-case
interpolation error across that range.

Using `--step 5` generates `65` points over the same range. In a local run, that
reduced the estimated worst-case interpolation error to about `0.105 C`.

That interpolation estimate is only about table interpolation. Real-world
accuracy is also affected by thermistor tolerance, resistor tolerance, ADC
noise, wiring, heat block coupling, and whether the assumed beta value and
parallel resistor value match the actual hardware.

Recommended starting points:

```bash
# Small and readable, already likely below the uncertainty of the parts.
./thermistor-tablegen.py

# Denser table, lower interpolation error, longer generated config.
./thermistor-tablegen.py --step 5

# Very dense table for experiments.
./thermistor-tablegen.py --step 2.5
```

## CLI Examples

Generate the default table:

```bash
./thermistor-tablegen.py
```

Generate to a specific output file:

```bash
./thermistor-tablegen.py -o printer.cfg-elegoon
```

Generate a denser table:

```bash
./thermistor-tablegen.py --step 5 -o printer.cfg-elegoon
```

Generate exactly `65` evenly spaced samples:

```bash
./thermistor-tablegen.py --samples 65
```

Change the temperature range:

```bash
./thermistor-tablegen.py --min-temp -20 --max-temp 350 --step 5
```

Change the thermistor model:

```bash
./thermistor-tablegen.py --r25 100000 --beta 3950 --reference-temp 25
```

Change the parallel resistor assumption:

```bash
./thermistor-tablegen.py --parallel-resistor 100000
```

Change the heater pullup assumption:

```bash
./thermistor-tablegen.py --pullup-resistor 4700
```

Change the generated sensor name:

```bash
./thermistor-tablegen.py --sensor-name elegoo_hotend_parallel_100k
```

## Interpreting Script Output

The script is intentionally verbose. It prints:

- The output file and generated sensor name.
- The thermistor model inputs.
- The equations being used.
- The temperature range and table density.
- Representative rows showing bare NTC resistance, compensated table
  resistance, and normalized ADC fraction.
- Estimated interpolation error across a dense set of points.

Example row:

```text
T= 160.000 C  R_ntc=1116.475 ohm  R_table=1104.148 ohm  adc_fraction=0.190234
```

Meaning:

- At `160 C`, the bare beta-model NTC would be about `1116.475` ohms.
- With a `100000` ohm parallel resistor, the MCU sees about `1104.148` ohms.
- With a `4700` ohm pullup, the normalized ADC fraction would be about
  `0.190234`.

## Practical Calibration Notes

The generated table is only as accurate as the model inputs.

The most important hardware assumptions are:

- The thermistor is actually close to a 100K beta 4300 NTC.
- The fixed parallel resistor is actually close to 100K.
- The pullup resistor used by the heater config matches the board's actual
  pullup.

If measured temperatures are consistently off after using this table, tune the
inputs based on measurements rather than editing individual table points by
hand. For example:

- If the bare thermistor model is wrong, adjust `--beta` or `--r25`.
- If the board shunt differs from the assumption, adjust `--parallel-resistor`.
- If the analog pullup differs, adjust `--pullup-resistor` and use the same
  value in the heater config.

After changing inputs, regenerate the whole table so the curve remains
internally consistent.

## Safety

Temperature sensor configs affect heater safety. After changing this table:

- Keep conservative `min_temp` and `max_temp` heater limits.
- Verify room-temperature readings before heating.
- Heat slowly and compare against an independent temperature reference if
  possible.
- Re-run PID or MPC calibration after changing the sensor model.

