# Item 4298, the attack deal: a human asks a computer to attack the third leader (`docs/LEADERS.md` §7.2, §8.2).
# run1575's line (`diplomacy4206.cmd`, run1482's three-seat lobby at TEAM_STYLE 0) through its 1081 Escape, then:
# meet the Russians (`treaties[2] & 1`, which `IFaceDiploNeg::update_visibility` needs before it offers the Mutual
# Attack check), reopen the negotiation window with the French, untick the alliance run1575 left ticked, tick the
# attack on the Russians at (676,551), give 5,000 each of the fourth and fifth goods rows (shift held: 500 a click),
# and PROPOSE. recon4298-4 (unfiled) is this file's rehearsal: `process_propose_attack 1 2 1131`, accepted on 1186.
900 meet who=1
901 resource who=0 all +100000
902 ally 1
905 @key 125
905 @key 52
905 @char 52
905 @key 13
905 @char 13
980 @key 129
985 @click 955 366
985 @key 13
985 @char 13
1000 @click 955 366
1000 @key 13
1000 @char 13
1015 @click 775 366
1020 @click 676 499
1026 @click 1206 564
1040 @click 770 499
1050 @click 1206 564
1080 @key 27
1080 @char 27
1081 @key 27
1081 @char 27
# 1085..1185: the attack deal (the first act's journal, "The next word").
1085 meet who=2
1090 @key 129
1092 @cursor 1400 1000
1110 @click 775 366
1115 @click 770 499
1118 @cursor 1400 1000
1130 @click 676 551
1133 @cursor 1400 1000
1145 @hold 16 1170
1146 @click 858 405
1147 @click 858 405
1148 @click 858 405
1149 @click 858 405
1150 @click 858 405
1151 @click 858 405
1152 @click 858 405
1153 @click 858 405
1154 @click 858 405
1155 @click 858 405
1156 @click 858 429
1157 @click 858 429
1158 @click 858 429
1159 @click 858 429
1160 @click 858 429
1161 @click 858 429
1162 @click 858 429
1163 @click 858 429
1164 @click 858 429
1165 @click 858 429
1172 @cursor 1400 1000
1185 @click 1206 564
1188 @cursor 1400 1000
