# Item 3437: run1034's take (item 3427, chapter twenty's lobby) on the click-free pool, its hand clicks as
# `@click` lines on the frames run1034's trace entered them (`tools/trace/click.h`).
0 !ai off
600 library who=0 1
604 add dock who=0 53,153
610 add chariot who=0 34,177
612 add chariot who=0 34,189
616 select 7 who=0
616 camera 34,189
# run1034 `c 96 1012`: 0/7's Auto Transport button -> Options::do_transport, issue_set_transport (626),
# and again (`process_set_transport` on 803 in run1034's dump)
626 @click 96 1012
802 @click 96 1012
