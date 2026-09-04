"""armyrecs.py <dump-name>... — the ARMY records of each frame block's first FULL DUMP, one line each."""
import sys
L = '/Users/rf-studio/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations/Logs/'
KEYS = ('army ', 'who ', 'status ', 'reg ', 'num_units ', 'num_captains ', 'num_standard ', 'city ',
        'hurry ', 'target_o ', 'target_who ', 'x ', 'y ', 'muster_x ', 'muster_y ', 'human_frame ', 'muster_angle ')
for name in sys.argv[1:]:
    print('==', name)
    fr = None; dumps = 0; rec = None
    with open(L + name, 'rb') as f:
        for l in f:
            t = l.decode('latin1').strip()
            if t.startswith('BEGIN FRAME'):
                fr = t; dumps = 0
            elif t == 'BEGIN FULL DUMP':
                dumps += 1
            elif t == 'BEGIN ARMY':
                rec = [] if dumps == 1 else None
            elif rec is not None and t.startswith(KEYS):
                rec.append(t)
                if t.startswith('muster_angle'):
                    print(fr, '|', ' '.join(rec)); rec = None
