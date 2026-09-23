# Golden record, chapter seven — THE CONTROL (item 578, run142).
#
# `chapter7.cmd` with its `0 !ai off` line deleted and nothing else changed:
# the pair docs/GOLDEN.md §11 asks for, kept as its own file so the two can
# never drift and the control is never an edit of the chapter. Its header,
# its recipe and its predictions are the chapter's; read them there.

# The Medieval age, which is where the Supply Wagon and the upgraded
# Merchant live, and late enough that a Caravan has somewhere to go.
600 library who=0 2

# Five civilians, each a different `think_*` arm, on their owner's ground
# and within reach of his capital.
610 add citizen who=0 24,160
615 add caravan who=0 28,160
620 add merchant who=0 24,164
625 add scholar who=0 28,164
630 add fur who=0 24,176
