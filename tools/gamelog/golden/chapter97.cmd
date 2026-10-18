# Golden record, chapter ninety-seven -- a ship's death object (item 3209, GOLDEN 128; parked 3208).
# Chapter 64's sea (region 70): two human Barks, who=1's Trireme, fishing boat and Bark. The Bark sunk on 800
# there left no death record in the dump (the capture ended at 820); run to 1500 so a sea unit's record, and
# its end (`DeathObj::inc_time@008d5240`, the animation alone for `type +0x218 == 1`), are printed.
#   RON_STAGE=ch5:chapter97:1500
#
0 !ai off

610 add bark who=0 60,180
612 add bark who=0 62,180
615 add trireme who=1 64,188
618 add fisher who=1 58,188
621 add bark who=1 76,192
