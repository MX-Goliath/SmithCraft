## 1 Help for Smith V4.1
SmithCraft is an independent Rust implementation of the Smith V4.1 workbench. Use the Contents tree to browse procedures, the Index to find a feature, or Search to search the manual. The viewer's Back, Forward, Home, Print and Options commands operate on these help topics.

The matching-network and S-Plot workspaces share the RF calculation engine. Values are recalculated when a point, component or tuning slider changes.

## 2 Overview
Smith Chart designs a matching network from a load toward a generator. S-Plot displays measured one-port or two-port data and calculates gain and stability. Mode switches between the two workspaces.

Use File to open a design or measurements. Design data are independent of the arrangement of the child windows. Drag a window caption to move it; use its corner to resize it. Window → Rearrange all restores the initial arrangement. Toolbars can float and dock.

## 3 Smith-Chart
The diagram maps impedance Z to the reflection coefficient Γ. The red grid is the impedance plane and the blue grid is the admittance plane. Cursor shows the impedance, admittance, Γ, VSWR, Q and return loss at the mouse position.

DP labels identify entered loads. TP labels identify the results of individual circuit elements. TP numbering continues after the last DP number. SP labels identify sweep results and start at 1. Select the start load before inserting components.

## 3.1 Menu and Toolbar
File contains project loading, saving, netlist export, data export and printing. Edit contains Undo, Redo and Copy to Clipboard. Mode selects Smith Chart or S-Plot. Tools contains Circles, Sweep, Tune and Settings. Zoom controls the diagram scale.

The Standard, Datapoints, Zoom, Mode and Elements toolbars have drag grips. Drag a grip to detach or move a group. Double-click its grip to return the group to the toolbar. Hover a symbol for its function.

## 3.2 Input of Datapoints
Mouse toggles point entry on the chart. Click a position and confirm its frequency. Key opens numeric entry. Values may represent impedance in ohms, admittance in siemens or the dimensionless reflection coefficient; Cartesian and polar entry are available.

Use S1 or S2 to open the s-Parameters selector. Select File loads Touchstone, CITI or EZNEC data. Click rows, Ctrl-click individual frequencies, or Shift-click a range, then confirm the selected measurements. The frequency unit selector converts the displayed unit without changing the physical frequency.

## 3.3 Input of Circuit Elements for the Network
First enter a load and select its Start DP checkbox. Choose a component in the Elements toolbar. Its locus shows the impedances that can be reached by varying that component. Move the mouse and click a permissible result to insert it.

Lines and stubs first request their characteristic impedance and effective relative permittivity. Lossy lines and composite RLC elements also provide numeric entry. Esc cancels a pending insertion. A component is applied to the current end of the network.

## 3.4 Circuit Elements
Available components are series and shunt R, L and C; a physical transmission line; open and shorted shunt stubs; an ideal transformer; a series element containing parallel RLC; and a shunt element containing series RLC.

All calculations use the exp(jωt) convention. Capacitors and inductors are ideal. The physical-line model includes conductor loss. A component's numerical entry and its plotted transformation use the same calculation.

## 3.4.1 Edit circuit element values
Double-click a component or its value in Schematic, or its TP row in Datapoints. Edit Element enables the fields applicable to that component. Enter a value and choose a unit. Draw recalculates the network while keeping the editor open. OK applies and closes the editor. Cancel restores the values from before the editor was opened.

Editing a component also recalculates all later transformed points and the current sweep.

## 3.4.2 Tune circuit element values, Tuning Cockpit
Open Tools → Tune, then select components in Schematic. A red rectangle identifies each selected component. Every slider initially spans 80–120% of its reference value. Reset Min/Max takes the current values as new range centers.

An RLC component has separate R, L and C sliders. A line has controls for electrical length and characteristic impedance. Remove takes a component out of the cockpit without deleting it from the network. Remove all clears the cockpit selection.

## 3.4.3 Capacitor (Serial or parallel element)
The capacitor is ideal. Enter its capacitance in pF, nF or µF. A series capacitor adds its impedance to the load; a shunt capacitor adds its admittance.

> ZC = 1 / (jωC); YC = jωC; ω = 2πf

Use a positive capacitance. Very small capacitances approach an open circuit in the shunt configuration. In the series configuration, increasing capacitance reduces the magnitude of its capacitive reactance.

## 3.4.4 Inductor (Serial or parallel element)
The inductor is ideal. Enter its inductance in pH, nH or µH. A series inductor adds inductive reactance. A shunt inductor adds its admittance to that of the load.

> ZL = jωL; YL = 1 / (jωL)

Use a positive inductance. The electrical effect varies with frequency, so both TP and sweep points change when the value is edited.

## 3.4.5 Resistor (Serial or parallel element)
Enter resistance in mΩ, Ω, kΩ, MΩ or GΩ. A series resistor increases the real part of impedance. A shunt resistor increases the real part of admittance.

> Zseries = Zload + R; Yshunt = Yload + 1/R

Resistance is positive. No thermal-noise model is added to the matching-network calculation; noise-figure circles use the supplied device noise parameters.

## 3.4.6 Physical Transmission Line (Serial element)
Specify characteristic impedance Z0, effective relative permittivity Er, length and attenuation. Length may be expressed as wavelength, electrical millimetres or physical millimetres. The fields are linked by propagation velocity c/√Er.

> θ = 2πf l√Er / c
> Zin = Z0 (ZL cosh γl + Z0 sinh γl) / (Z0 cosh γl + ZL sinh γl)

Attenuation is specified in dB per physical metre at the reference frequency and scales with √f. Use effective permittivity for planar transmission lines.

## 3.4.7 Open ended Transmission Line, Open Stub
An ideal open-ended stub is connected in shunt with the load. Enter its characteristic impedance and electrical length. Er links physical and electrical lengths.

> Ystub = j tan(θ) / Z0

The end of the stub is electrically open. Lengths differing by half a wavelength have the same lossless input admittance at the reference frequency. Values near a quarter wavelength are numerically very large in admittance.

## 3.4.8 Shorted Transmission Line, Shorted Stub
The shorted stub is an ideal shunt transmission-line section terminated in a short circuit. Its input admittance depends on its electrical length and characteristic impedance.

> Ystub = −j / (Z0 tan θ)

Lengths differing by half a wavelength are equivalent at the reference frequency. A quarter-wave shorted stub approaches an open circuit at its input. The drawn short belongs to the far end of the stub.

## 3.4.9 Transformer (Serial element)
The transformer is ideal. Its value n is a voltage or turns ratio and must be positive. The impedance ratio is the square of n.

> Zin = n² Zload

Both real and imaginary impedance are scaled by n². Transformer loss, leakage inductance and winding capacitance are not modeled.

## 3.4.10 R-L-C parallel connected (Serial element)
This series circuit element consists internally of R, L and C in parallel. Enter R, L and C independently. The resulting impedance is added to the load.

> ZRLC = 1 / (1/R + jωC + 1/(jωL))

All three values can be adjusted in Tune. At parallel resonance the reactive terms cancel and the impedance approaches R.

## 3.4.11 R-L-C serial connected (Parallel element)
This shunt circuit element consists internally of a series RLC branch. The admittance of that branch is added to the load admittance.

> ZRLC = R + jωL + 1/(jωC)

All three values can be tuned independently. At series resonance the branch impedance approaches R. The branch remains connected between the network node and the return conductor.

## 3.5 Edit Datapoints
Double-click an entered point on the chart or its row in Datapoints to edit Z, Y or Γ and its frequency. The associated network is recalculated. Double-clicking a transformed point opens the corresponding component editor.

The selected Start DP is the load used by the component chain. Choose it before adding components. A frequency sweep treats that load impedance as constant; a datapoint sweep uses each measured load at its own frequency.

## 3.6 Zoom-Function
Zoom in, Zoom out and Zoom 1:1 scale the diagram around its center. Their shortcuts are Ctrl+1, Ctrl+2 and Ctrl+3. The toolbar offers the same three operations.

Zoom changes only presentation; it does not alter the points or component values. Diagram mouse coordinates and Cursor are evaluated at the current scale.

## 3.7 Undo and Redo
Undo reverses design changes to points and components. Redo reapplies the most recently undone design change. A new design change clears the redo branch. Right-clicking the diagram also invokes Undo.

Circle visibility, circle highlighting and global presentation settings are independent of the design history. They are retained when a component edit is undone. Undo and Redo are disabled when their respective histories are empty.

## 3.8 Sweeps
Open Tools → Sweep or click S… in the Mode toolbar. Sweep over datapoints transforms every entered load at that point's frequency. Sweep over frequency transforms one fixed start-load impedance through the selected frequency interval.

For frequency sweep, set start frequency, stop frequency and step, using the unit selectors. Number of points is calculated from the interval and step. Draw updates the plot and keeps the dialog open. Remove Sweep or Clr removes the sweep. Green curves identify sweep results; orange circles identify sampled frequencies.

## 3.9 Circles
Tools → Circles opens Q, Gain, VSWR, Stability and Noise Figure tabs. Q and VSWR have predefined values and a list for additional values. Gain, stability and noise circles can use manually entered parameters or measurements imported from an S-parameter file.

Gain, stability and noise circles are identified as G1…, S1… and N1…. Circles lists their parameters and controls visibility and highlighting. A highlighted circle is drawn continuously; other device circles use dashed lines.

## 3.9.1 Constant Q-Circles
Q describes the magnitude of reactance relative to resistance. Choose predefined values or insert another nonnegative value. Each Q contour includes inductive and capacitive branches.

> Q = |X/R|

Q contours are geometric aids for a matching design. They do not introduce loss or limit the Q of the ideal inductors and capacitors. Select all and Clear all affect the predefined contour values.

## 3.9.2 Constant Gain Circles
Available gain uses the source/input plane. Operating gain uses the load/output plane. The unilateral approximation neglects reverse transmission. Enter S11, S12, S21 and S22 in polar, Cartesian or dB form, or import a frequency from measured data.

Enter the gain in dB and choose Draw. The stability factor K and available MAG/MSG help identify feasible gain values. A gain exceeding the attainable maximum does not define a real circle.

## 3.9.3 Constant VSWR Circles
Constant VSWR corresponds to constant |Γ|, so these circles are centered at the origin of the reflection-coefficient plane. Choose a predefined ratio or add another value of at least 1.

> |Γ| = (VSWR − 1) / (VSWR + 1)

VSWR = 1 is the matched point. As VSWR increases, the circle approaches the outer boundary of the passive Smith chart.

## 3.9.4 Stability-Circles
Choose the input or output plane and provide the four S-parameters. The input-plane circle is the boundary where the output reflection magnitude becomes 1. The output-plane circle is the boundary where the input reflection magnitude becomes 1.

The stable side is found by testing a point, including the matched termination Γ=0. Details identifies the stable inside or outside region. K and μ are shown for the selected device frequency.

## 3.9.5 Constant Noise Figure Circles
Provide the minimum noise figure, optimum source reflection coefficient and equivalent input noise resistance. Rn may be entered normalized to Z0 or in ohms. Polar and Cartesian forms are available for Γopt.

The noise figure to show must be no smaller than NFmin. At NFmin the circle collapses to Γopt. Measured noise records may be selected from imported data. Noise-figure values use dB; reflection coefficient and normalized Rn are dimensionless.

## 3.10 Settings
Smith-Chart controls the Z/Y grids, chart labels, point labels, contour labels, point connections, line widths and colors. It also provides default reference impedance, point frequency and schematic orientation.

S-Plot has its own grid and marker switches and colors for MAG, MSG, K, μ and return loss. General selects the startup workspace. Color swatches open the Windows color dialog. Restore defaults restores the reference palette. Settings are saved independently of project data.

## 3.11 Print the Smith-Chart
File → Print and Print Preview open the same item selector. Choose the diagram layers and the schematic, point table or circle table to include. Preview offers page navigation, whole-page or page-width fitting and numeric zoom levels.

Print opens the Windows common printer dialog. Select a printer, copies and page range there. Microsoft Print to PDF may be used when available. Preview and printing share vector pages, so the selected content is identical in both.

## 3.12 Copy to Clipboard
Edit → Copy to Clipboard opens a surface selector. Choose the Smith chart, schematic, datapoints, circles, an S-parameter chart, source information, gain table, gain graph or return-loss graph.

OK captures the selected work surface as an image in the clipboard. Resize or expand a work window before copying if a larger image is needed. A minimized window should be restored before capture.

## 3.13 Shortcuts
Ctrl+N creates a design; Ctrl+O opens data or a project; Ctrl+S saves. Ctrl+Z and Ctrl+Y are Undo and Redo. Ctrl+M switches workspaces. Ctrl+T opens Circles; Ctrl+E opens Settings. Ctrl+1, Ctrl+2 and Ctrl+3 control zoom. Ctrl+F1 opens Help.

Esc cancels a pending component insertion or closes the current editor. Right-click on the diagram invokes Undo. Ctrl and Shift support multiple selection in the S-parameter import table.

## 3.14 Save Netlist
File → Save Netlist writes a Touchstone-style circuit netlist. The circuit runs from load toward generator, with the return node numbered 0. Capacitors, inductors and resistors use CAP, IND and RES. Composite branches use PRLC or SRLC; transformers use TRF.

Transmission lines use TLINP with electrical length in millimetres, effective permittivity and attenuation in dB per physical metre. Open stubs terminate on distinct floating nodes. The frequency section uses the start point's frequency.

## 3.15 Export Data
File → Export writes datapoints or circle parameters as text. Choose no delimiter, semicolon or comma for the target program. Point export includes the start flag, point number, resistance, reactance, Q and frequency in Hz.

Circle export includes G/S/N identifiers and the underlying gain, stability or noise parameters. SVG export provides a scalable diagram image. Saving a project retains editable design data; exporting a table is for subsequent analysis.

## 4 S-Plot
S-Plot reads one-port and two-port measurement files. S11 and S22 use Smith charts. S12 and S21 use polar charts with an amplitude scale appropriate to the measurements. Red dots mark frequencies and a green square marks the selected record.

Info Window shows the file contents. Gain table lists MAG, MSG, K, μ and reverse transmission. Gain graph and Return Loss graph share the selected frequency with the S charts and table. Use Diagram for logarithmic or linear frequency spacing and marker information.

## 4.1 Export
The Export menu converts measured S-parameters to H, Z, Y or A/ABCD parameters. CITI-to-S writes Touchstone S-parameter data. S11 or S22 to Smith Chart opens the import selector for those loads.

Conversions use the measurement file's reference impedance. The stored order is S11, S21, S12, S22. A one-port file supplies only S11; two-port operations require a complete matrix.

## 4.1.1 Export H-, Z-, Y- and A-Parameter to File
Choose the destination parameter family and whether to normalize its entries. The current S matrix is converted independently at each frequency. H is the hybrid matrix; A is the chain or ABCD matrix.

Physical Z and Y carry ohm and siemens units. The hybrid and chain matrices mix units. The Touchstone version 2 header identifies physical matrices explicitly. Normalized output uses the corresponding reference-impedance scaling. Files that would require a singular matrix inversion are rejected.

## 4.1.2 Export CITI to S-Parameter Touchstone File
Load a CITI dataset in S-Plot, then choose CITI-to-S. The exported Touchstone file contains the loaded S matrix at every frequency. Supported noise records are retained when exporting S data.

CITI can contain several packages or independent variables; this implementation uses the first frequency package. The source information window makes the imported package available for inspection.

## 4.1.3 Export S11 or S22 to Smith Chart
Choose S11 or S22 to Smith Chart in Export. Select the desired rows. Ctrl-click toggles individual rows; Shift-click selects a frequency interval. Select all and Unselect all are available in the table's context menu.

Confirm to create editable DP loads in Smith Chart. Each load retains its frequency. After choosing a start DP and creating a network, datapoint sweep evaluates the network across those measured loads.

## 4.2 Print S-Plot
Print and Print Preview let you choose S11, S12, S21, S22, Info Window, Gain table, Gain graph and Return Loss graph. Only the selected sections are included.

K and μ are dimensionless and use the right axis of the gain graph. MAG and MSG use its left dB axis. Return-loss curves use their own dB scale. These same scales are used in preview and vector output.

## 4.3 Copy to Clipboard
Choose a work surface in Copy to Clipboard. The four S charts, source information and all three gain/return-loss surfaces can be captured separately.

The selected frequency marker is included. Change the selection in any linked S chart or Gain table before copying to capture the desired frequency.

## 5 Smith-Chart Basics
The chart is a geometric view of complex impedance and admittance. It maps the passive half-plane into a unit disk in reflection-coefficient coordinates. Matching can be studied by component transformations, stability boundaries, gain circles and noise circles.

The following topics collect the principal equations used by the RF engine. They describe a linear network model and use a real positive reference impedance.

## 5.1 Smith-Chart Construction
Normalize impedance by z=Z/Z0. The transformation and its inverse are:

> Γ = (z − 1) / (z + 1)
> z = (1 + Γ) / (1 − Γ)

Constant resistance maps to circles centered on the real Γ axis. Constant reactance maps to arcs. The normalized admittance grid is obtained by rotating the impedance grid by 180 degrees. The matched impedance Z0 maps to Γ=0.

## 5.1.1 Stability Circles
For a two-port define Δ=S11S22−S12S21. Stability can be characterized by K together with |Δ|, or by μ criteria. Unconditional stability requires all passive source and load terminations to be stable.

> K = (1−|S11|²−|S22|²+|Δ|²) / (2|S12S21|)

The circle boundary alone does not identify the stable side; test a termination. A degeneracy may produce a straight-line boundary or no finite circle.

## 5.1.1.1 Source Stability Circles
The source-plane boundary corresponds to |Γout|=1 while ΓS varies. Its center and radius follow from the output reflection equation.

> Γout = S22 + S12S21ΓS / (1−S11ΓS)

At ΓS=0, the output reflection coefficient is S22. This gives a direct test for which side of the boundary is stable. The displayed circle may extend beyond the unit chart.

## 5.1.1.2 Load Stability Circles
The load-plane boundary corresponds to |Γin|=1 while ΓL varies.

> Γin = S11 + S12S21ΓL / (1−S22ΓL)

At ΓL=0, use S11 to test the stable side. The source and load circles are different geometric objects and refer to different termination planes.

## 5.1.2 Constant Gain Circles
Several gain definitions are useful. Transducer gain compares power delivered to the load with power available from the source. Available gain assumes an output match; operating gain uses accepted input power. Their constant-gain loci need not coincide.

Use the definition appropriate to the termination plane. Device stability and the attainable maximum gain constrain which gain contours have real solutions.

## 5.1.2.1 Transducer Power Gain
Transducer gain includes both source and load mismatch and reverse transmission.

> GT = |S21|²(1−|ΓS|²)(1−|ΓL|²) / |(1−S11ΓS)(1−S22ΓL)−S12S21ΓSΓL|²

The expression reduces to |S21|² for matched source and load. When converting to dB, use 10 log10 for a power ratio, or equivalently 20 log10 for an amplitude ratio.

## 5.1.2.2 Operating Power Gain
Operating gain is referenced to the power accepted at the input. It depends on ΓL and can be used for an output-plane matching construction.

> GP = |S21|²(1−|ΓL|²) / ((1−|Γin|²)|1−S22ΓL|²)

The input reflection coefficient itself depends on ΓL when reverse transmission is nonzero. This dependence is retained in bilateral gain circles.

## 5.1.2.3 Maximum Available Gain MAG and Maximum Stable Gain MSG
For an unconditionally stable two-port, MAG is obtained at simultaneous conjugate matching. A potentially unstable device instead uses MSG as a stability-related gain measure.

> MAG = |S21/S12| (K−√(K²−1))
> MSG = |S21/S12|

These are power gains. Gain table separates the applicable MAG and MSG columns at each measured frequency. Degenerate zero reverse transmission is handled as a limiting case.

## 5.1.2.4 Available Power Gain
Available gain compares output-available power with source-available power. It depends on ΓS, with an output match assumed.

> GA = |S21|²(1−|ΓS|²) / ((1−|Γout|²)|1−S11ΓS|²)

Available-gain circles use the source/input plane. They are useful when evaluating source matching together with noise-figure circles.

## 5.1.2.5 Unilateral Transducer Gain
The unilateral approximation sets S12=0. Under this approximation, source mismatch and load mismatch factors separate. It can provide an initial estimate when reverse transmission is sufficiently small.

> GTU = |S21|²(1−|ΓS|²)(1−|ΓL|²) / (|1−S11ΓS|²|1−S22ΓL|²)

The approximation is a selectable gain-circle mode. The measured bilateral stability factors still describe the original S matrix.

## 5.1.3 Simultaneous Conjugate Match
Simultaneous conjugate matching requires ΓS to be the conjugate of Γin and ΓL to be the conjugate of Γout. Reverse transmission couples the two conditions.

For an unconditionally stable device, choose the solutions inside the unit disk. The resulting maximum available gain is MAG. A solution outside the passive chart would require an active termination and is not a passive matching-network target.

## 5.1.4 Constant Noise Figure Circles
Use linear noise factors F and Fmin in the noise equation, even though the interface accepts dB. Let rn=Rn/Z0.

> F = Fmin + 4 rn |ΓS−Γopt|² / ((1−|ΓS|²)|1+Γopt|²)

For fixed noise figure this defines a source-plane circle. Its center approaches Γopt and its radius approaches zero as F approaches Fmin. Negative radius-squared values do not describe a real noise circle.

## 6 File format info
Projects and measurement files serve different purposes. Smith XML stores an editable matching design; Touchstone and CITI store network measurements. EZNEC source-impedance data are imported as one-port measurements.

The native .smithcraft format retains the complete Rust project. Original .xmlsc import and export use the Smith V4.1 schema. Smith keeps its display settings and reference impedance globally; those are not fields in the original XML design schema.

## 6.1 Touchstone
One-port and two-port files support real/imaginary, magnitude/angle and dB/angle records. Frequency units in the option line are honored. Two-port data use the order S11, S21, S12, S22.

S, Z, Y, H and ABCD matrices can be read or exported. Supported version 2 files contain full one-port or two-port matrices and equal real port references. Unsupported matrix arrangements are rejected explicitly. Noise records contain Fmin, Γopt magnitude/angle and normalized Rn.

## 6.2 CITI
CITI separates variable definitions from data arrays. The reader resolves the frequency variable and identifies the S-matrix arrays by name. Data may be represented in real/imaginary or magnitude/angle form.

The first frequency package is imported. Available minimum-noise, optimum-reflection and noise-resistance arrays are associated with their frequencies. Inspect Info Window after import when a file contains several packages or variables.

## 6.3 EZNEC
The EZNEC impedance reader imports source 1 from the source-impedance text output. Each frequency and its complex impedance are converted to a one-port reflection coefficient using the selected reference impedance.

Use the resulting S11 data as measured loads in Smith Chart. Datapoint sweep then evaluates a matching network against the frequency-dependent antenna impedance.

## 7 References
The reference workbench is Smith V4.1 by Fritz Dellsperger. SmithCraft uses an independent Rust implementation and independently authored help. No original executable or library is required to run this help or the calculation engine.

The equations in this manual are standard linear microwave-network relations. Their numerical implementation is in src/rf.rs. Tests cover impedance/reflection conversion, matching, lines, two-port matrix conversions, gain, stability and noise circles.

## 8 Implementation and License
SmithCraft source is licensed under the MIT license included with the repository. Its version and independent implementation are identified in About Smith. The original Smith application's licensing terms and registration system apply to that original product, not to this implementation.

The Rust workbench imposes no original-demo point or component limit. Standard operating-system dialogs and the HTML Help viewer are supplied by Windows.

## 9 Feedback
When reporting a discrepancy, retain the smallest project or measurement file that demonstrates it. Record the input values, selected frequency, requested operation and expected result. A screenshot is useful for layout issues; the project file is needed to reproduce a calculation.

For a future interface redesign, the classic workspace, dialog and toolbar modules can be changed separately from the RF model and file-format code.

## 10 History
Version 0.1 introduced the Rust RF engine, project model, import/export and initial interface. Version 0.2 introduced the classic MDI workspace, independent dialog windows and native printing.

Version 0.3 extends reference-interface correspondence: Smith-compatible XML output, original point and circle numbering, table behavior, frequency-step Sweep, Windows color selection, complete RLC tuning and a native compiled help system.
