# SmithCraft

A native Windows application written in Rust for working with Smith charts and S-parameters. Version 0.3.4 recreates the classic Smith V4.1 interface: a gray workspace, movable internal windows, menus, five toolbar groups, component symbols, and separate dialogs. Component icons were reconstructed from a user-provided screenshot; the original application's source code and libraries are not used.

The Elements toolbar follows the original order: series C, L, R; transmission line; transformer; a parallel RLC network connected in series; shunt C, L, R; open- and short-circuited stubs; and a series RLC network connected in shunt. Each button uses the same component type for its icon, tooltip, and action.

Numeric values in the interface and printed reports are displayed with two decimal places. Calculations, saved projects, and numeric exports retain full precision.

## Running

The built executable is `dist/SmithCraft.exe`. No installation, .NET, or internet connection is required to run it.

```powershell
.\dist\SmithCraft.exe
.\dist\SmithCraft.exe --demo
.\dist\SmithCraft.exe .\examples\amplifier.s2p
```

A normal launch opens an empty project, as in Smith. To demonstrate a 25-to-50-ohm match, use `--demo` or open `examples/l-match.smithcraft`. At 1 GHz, a series inductance of 3.978873577 nH and a shunt capacitance of 3.183098862 pF produce an input impedance of 50 + j0 ohms. The green curve on the Smith chart is the frequency sweep, covering 500–1500 MHz in the example; remove it with **Tools → Remove Sweep** or **Clr**. In the Gain graph, green represents K: K and μ use a separate right-hand scale, while MAG and MSG use the left-hand scale in dB.

## Features

- Scalable Smith chart, Z/Y grids, arbitrary Z0, labels, and configurable colors and line widths.
- Point entry by mouse or numeric input: Z, Y, and Γ in Cartesian or polar form. Frequency and the starting point are set separately.
- Series and shunt R/L/C, lossy transmission lines, open- and short-circuited stubs, ideal transformers, parallel RLC networks connected in series, and series RLC networks connected in shunt.
- Numeric editing, component value selection on the chart, enabling/disabling and reordering elements, with immediate recalculation of the schematic and intermediate impedances.
- Tune window: component selection on the schematic, vertical value sliders covering 80–120%, range reset, frequency sweeps, and sweeps over measured data points.
- VSWR circles, Q contours, stability circles, available, operating, and unilateral gain circles, and noise figure circles. Separate visibility and highlighting controls.
- Cursor readout: impedance, admittance, reflection coefficient, VSWR, Q, return loss, and frequency.
- S-Plot: Touchstone 1.x and a supported subset of 2.x, CITI, and EZNEC; four S-parameter plots, MAG/MSG, K, μ and μ′, return loss, a numeric table, linear/logarithmic axes, and linked selection.
- Import all or selected S11/S22 points into the Smith chart; transfer noise data and S-parameters to the circle dialogs.
- Export S/Z/Y/H/ABCD data in normalized form or physical units, point and circle CSV files, SVG, and CKT netlists; copy tables and chart images to the clipboard.
- Native Print Preview with page and zoom selection, the Windows printer dialog, vector printing, and Microsoft Print to PDF. Print options control which pages and chart layers are included.
- Save and open `.smithcraft` projects, remember window layouts and interface settings, access recent files, undo/redo, receive prompts for unsaved changes, and open files by drag and drop.
- Read and write Smith `.xmlsc` projects: points, all 12 element types, circles with their original parameters, and sweeps. Reading was checked against 22 published examples; an exported project containing a circuit and ten circles was verified by opening it in the original Smith application.

## Usage

1. **Key…** — enter the load and its frequency. In **Mouse** mode, click the chart and confirm the frequency.
2. Select the starting point with the **Start** control.
3. Click an element on the toolbar and choose its value along the chart trajectory. For transmission lines, set the characteristic impedance and Er first. Press `Esc` to cancel placement.
4. Double-click a component symbol to edit it. In **Edit Element**, change the value and click the green **Draw** button to recalculate, or **OK** to apply and close. **Cancel** restores the original values.
5. **Sweep** performs a frequency sweep with a constant load or transforms all measured points at their respective frequencies. **Tune** adjusts component values and the entire sweep together.
6. **Open S-Parameter** opens measurements in S-Plot. **S1** / **S2** open the frequency selection table for importing S11 / S22 into the Smith chart.

Numeric fields accept normal keyboard input, including a decimal comma. Units are selected beside each field. Transmission line lengths can be specified as fractions of a wavelength at the reference frequency or in millimeters; effective relative permittivity is taken into account. Attenuation is specified in dB/m at the reference frequency and scales as √(f/fref).

**Window → Rearrange all** restores the initial window layout. Internal windows can be moved by their title bars, minimized, maximized, and resized using the lower-right corner. Toolbar groups can be detached by dragging their vertical grips; double-click a grip to return the group to its original position.

| Shortcut | Action |
|---|---|
| Ctrl+N / O / S | New / open / save |
| Ctrl+Z / Y | Undo / redo |
| Ctrl+M | Chart / S-Plot |
| Ctrl+T / E | Circles / settings |
| Ctrl+1 / 2 / 3 | Zoom in / zoom out / 1:1 |
| Ctrl+F1 (also F1) | Help |
| Esc | Cancel chart input |
| Right-click the chart | Cancel |

## Building and testing

The Windows installer, portable ZIP, and Linux AppImage are built using dedicated scripts and GitHub Actions. Commands, dependencies, and package checks are documented in [BUILDING.md](BUILDING.md).

A recent Rust toolchain is required. For GNU builds with `stable-x86_64-pc-windows-gnu`, the build script uses a local portable w64devkit from `.tools/`. Standard Rust MSVC builds with Visual Studio Build Tools are also supported.

```powershell
.\build.ps1 -Test -Release
.\build.ps1 -Release -Run -Demo
```

In an environment with a configured compiler:

```text
cargo test --locked
cargo build --release --locked
```

The calculation engine is tested for L-network matching, quarter- and half-wave lines, losses, transformers, stability, noise circles, matrix round trips, and project persistence. If Smith is installed, the tests read all files in its `S-parameter` directory; if `.reference/examples` is present, they compare intermediate impedances against all 22 original example projects. Third-party examples and help files are kept only in the ignored `.reference/` directory.

## Compatibility and limitations

- The interface layout and controls were checked against the running Smith application and its help. Rendering was reimplemented in egui, so system antialiasing and some icons differ. Color and file selection use native Windows dialogs. The independently written help contains 62 topics in native HTML Help, with a table of contents, index, search, history, and printing. The About dialog identifies the application as an independent Rust implementation. Printed output splits selected charts and tables across separate A4 pages.
- Projects are saved in the compatible Smith XML `.xmlsc` format by default. As in the original, this format stores the project, while Z0 and appearance come from global application settings. `.smithcraft` stores the full JSON project, including settings, measurements, and disabled components. The older SmithCraft XML wrapper can still be read. Legacy binary `.smc` files are not supported.
- Touchstone: one or two ports, a full matrix, and equal real reference impedances. Sparse/mixed matrices, unequal or complex Z0 values, and networks with more than two ports are rejected. Physical Z/Y/H/ABCD data is exported with a version 2.0 header; normalized data uses 1.x. ABCD uses the A,C,B,D order.
- EZNEC: source 1 is imported. CITI: the first frequency variable, S-matrix, and available NFmin/Sopt/Rn data are imported; multidimensional datasets are not supported.
- Components are ideal and the model is linear; finite inductor and capacitor Q is not modeled separately, as in the original. Physical values and frequency must be positive.
- A manually entered load is treated as constant during a frequency sweep. For frequency-dependent loads, use a sweep over measured data points.
