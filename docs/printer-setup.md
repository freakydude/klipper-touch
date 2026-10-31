# Slicer and macros

FreakyMate works with any Klipper printer as it is. A few settings in the slicer and a few common macros let it show
and offer more.

## Slicer settings

**Thumbnails.** FreakyMate shows the slicer's thumbnail in the file list and on the print page, in a square. Of the
thumbnails in a file it takes the sharpest one in that square (the longest short side; on a tie the squarer one), so a
square PNG fills it without borders. A small one for the list and a square one for the print page are enough, for
example `32x32/PNG, 300x300/PNG` in the thumbnail setting of PrusaSlicer or OrcaSlicer. A file without any thumbnail
shows only its name in the list and a file symbol on the print page.

**Layers.** Klipper only knows the layers when the slicer tells it. With these lines the print page shows the current
layer and the number of layers as Klipper reports them, and the [layer pause](#pause-at-a-layer) becomes available.
Without them it shows no current layer, and the number of layers only as an estimate from the object and layer height
(with a `~`). The heights above the layers need no setup: the current one is the Z of the file
(`gcode_move.gcode_position`, without Z offset and bed mesh, the lowest of the last two seconds so a Z hop does not
count; it stays during a pause), the maximum is the object height of the file. In PrusaSlicer, SuperSlicer and OrcaSlicer (Printer settings → Custom G-code):

| Field                    | Line                                                  |
| ------------------------ | ----------------------------------------------------- |
| Start G-code (first line) | `SET_PRINT_STATS_INFO TOTAL_LAYER=[total_layer_count]` |
| After layer change G-code | `SET_PRINT_STATS_INFO CURRENT_LAYER={layer_num + 1}`   |

Other slicers use other placeholders; the [Mainsail](https://docs.mainsail.xyz/slicers/prusaslicer/) and
[KlipperScreen](https://klipperscreen.readthedocs.io/en/latest/Quicktips/) documentation list them.

**Objects.** To exclude single objects during a print, Klipper needs `[exclude_object]` in `printer.cfg` and the file
needs the objects labelled: turn on the slicer's option to label objects for Klipper (it writes
`EXCLUDE_OBJECT_DEFINE`), or let Moonraker do it with `enable_object_processing: True` in its `[file_manager]` section.
When the file defines the outlines of the objects, the Objects dialog draws them on a map of the bed; otherwise it
lists them by name.

**Matching colours (just for fun).** The screenshots show a benchy in FreakyMate's accent colour: set the filament
colour in the slicer to `#FD7E14` (RGB 253, 126, 20) and the thumbnail matches the screen.

## Macros

FreakyMate calls macros by name with their documented parameters and makes no assumptions about what they do inside.
Where a macro is missing, its button does not appear.

### Load and unload filament

Without Happy Hare the **Load** and **Unload** buttons of the Filament page run the macros `LOAD_FILAMENT` and
`UNLOAD_FILAMENT`, when the printer has them. FreakyMate calls them without parameters, so they need defaults for
everything they take. KlipperScreen's documentation has
[example macros](https://klipperscreen.readthedocs.io/en/latest/macros/) for both. With Happy Hare the buttons load
and unload through the MMU instead.

### Pause at a layer

A tap on the layer tile of the print page plans a pause. One counter starts at the next layer and shows below each
layer the height it will have (an estimate with a `~`: the current height plus the slicer's layer height per layer,
so a variable layer height makes it wrong). FreakyMate pauses at the next layer with `SET_PAUSE_NEXT_LAYER` and at a
later one with `SET_PAUSE_AT_LAYER`, both from Mainsail's
[`client.cfg`](https://github.com/mainsail-crew/mainsail-config) (mainsail-config), which most Mainsail installations
include already. The tile offers the pause while a print runs, at least one of the macros exists and the slicer
reports the layers (see [Layers](#slicer-settings) above). The dialog also takes a planned pause back.

The plan itself lives in the variables `pause_next_layer` and `pause_at_layer` of mainsail-config's
`SET_PRINT_STATS_INFO`. FreakyMate reads them, so a pause planned here or in another client shows on the layer tile
(the layer, or `next`, below the pause icon) and in the dialog. With a different `SET_PRINT_STATS_INFO` that lacks
these variables, the plan is not shown and the dialog always offers to take it back.
