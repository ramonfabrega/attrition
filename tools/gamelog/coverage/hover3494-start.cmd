# Item 3494: the golden start (map 14) at cover=1 on the click-free pool, the cursor pinned on the food counter at
# 600 (run1082's point, game (70,33), parked 3463a) and held there to the take's end, with `--hover`: the tracer
# re-sends the pinned `OnMouseMove` at `Scene::describe_mouseover@008b47e0`'s entry (`tools/trace/hover.h`), after
# `Scene::render` has zeroed `mouseover->type`. Predicted: `MouseOver::describe` case 0x200 ->
# `Options::describe(OPTION_GOOD, food)` -> `Game::is_turbo_resources` and `LeaderData::resource_cap_get`.
#
# The twin (run1130) is this file and `--hover` at cover=0: every input shared.
600 @cursor 70 33
