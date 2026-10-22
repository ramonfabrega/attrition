# Item 3455: the coverage pair's lobby at cover=1 on the click-free pool, AI on: the human's ship right-clicked onto
# the human's own Dock (docs/VISION.md §8): `Console::execute_at_cursor:2186-2195` -> `ObjectData::is_dock@004711e0`,
# the garrison order when the target is a Dock and the selection holds a sea unit.
# The Dock and the sea cell are 3446's (carrier3446.cmd): `add dock who=0 13,37` was 0/2007 and the next unit 0/14.
# `camera 13,37` draws the Dock's tile at the view's centre, (960, 445) (run1032's mapping, 3443's trade click).
# The first unshared input is 600; `samegame.py --from 600` against run672 (blocks 577..833).
600 add dock who=0 13,37
602 add trireme who=0 6,37
606 select 14 who=0
606 camera 13,37
615 @rclick 960 445
