# Item 3479: the golden start (Great Lakes, map 14) at cover=1 on the click-free pool, `--bindkey
# SYSTEM_TOGGLE_PARAM_CONSOLE=123`: the param console opened by F12 and two field edits sent by Return.
# `keys 1` is console command 17 (`run_cmd` case 0x11 sets `cheat_keys`), which `Console::exec_keymap` case 0x1e
# tests before `ParamRegisterWindow::exec`. `exec` is `set_modal` with flags 0, a nested modal loop entered inside
# the `@key 123` line, so every later input is an `@after` line queued before it (3445).
# Return reaches `ParamRegisterWindow::on_key_down` from any child (no control overrides slot +0x11c) and calls
# `on_change_edit_box@009345e0`, which calls slot +0xc of the current object. The window opens on Lights/Sun
# (every registration leaves cur_cat = cur_obj = 0). Its controls are built once, for Sun, and never again:
# `process@00934c90` rebuilds only while the control array is empty (0x934cd2). So a Return is safe only on an
# object with no edit-box (plain int) parameter. Nuke (an enum, dwords, floats) has none: its Return reaches
# `Nuke::on_field_change@00925cb0`. BorderSpline (ten ints) and CircleRender (three) hand one of Sun's Scrolls to
# `EditBox::get_text`, and recon3479-3 faulted there. Points come from recon3479-2's window photograph: the
# category combo at y 31, its pulldown rows 18 px apart from Farms at 47, Nuke the sixth (137).
0 !ai off
600 keys 1
602 @after 4000 key 13
602 @after 8000 click 380 31
602 @after 12000 click 380 137
602 @after 16000 key 13
602 @after 20000 key 27
602 @key 123
