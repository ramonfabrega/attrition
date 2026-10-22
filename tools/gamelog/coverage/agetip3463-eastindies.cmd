# Item 3463: the first pair's East Indies lobby (map 18, DIFFICULTY 0, AI on, seed 12345) on the click-free pool,
# `--ffwd-minute 11`, so the game fast-forwards to 9900 and draws from there (a hover needs the drawn scene).
# Leader 1's build-order script ends on 9977 (`prod_script_run` 1 -> 0, run139), and `production_ai` reaches step
# 4, `research_techs`, on 9979 (run29, recon3463-2). By then the human is three ages ahead (`age who=0 3`, the
# Gunpowder Age) and the computer holds 100000 of every good, so the age techs pass the knowledge gate and, at
# difficulty 0, ask `Leaders::best_human_age_stamp` (AI §2.14's human pacing).
# The food counter is the resources window's second row, at game (70,33) on a window-only photograph of
# recon3463-2 at 9930 (1920x1080, no title bar). `@cursor` sends one `OnMouseMove`; `Scene::render` clears
# `mouseover->type` every drawn frame, so the cursor is moved on the counter every frame for thirty frames, then held
# still there to 9990, and then put back on the resting point.
9905 age who=0 3
9906 resource who=1 all +100000
9910 @cursor 70 33
9911 @cursor 71 33
9912 @cursor 70 33
9913 @cursor 71 33
9914 @cursor 70 33
9915 @cursor 71 33
9916 @cursor 70 33
9917 @cursor 71 33
9918 @cursor 70 33
9919 @cursor 71 33
9920 @cursor 70 33
9921 @cursor 71 33
9922 @cursor 70 33
9923 @cursor 71 33
9924 @cursor 70 33
9925 @cursor 71 33
9926 @cursor 70 33
9927 @cursor 71 33
9928 @cursor 70 33
9929 @cursor 71 33
9930 @cursor 70 33
9931 @cursor 71 33
9932 @cursor 70 33
9933 @cursor 71 33
9934 @cursor 70 33
9935 @cursor 71 33
9936 @cursor 70 33
9937 @cursor 71 33
9938 @cursor 70 33
9939 @cursor 71 33
9940 @cursor 70 33
9990 @cursor 700 300
