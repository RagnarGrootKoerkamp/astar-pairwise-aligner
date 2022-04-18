import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np
from IPython.display import display
from pathlib import Path
import seaborn as sns
import math
#%matplotlib inline

pd.set_option('display.max_rows', 500)
pd.set_option('display.max_columns', 500)
pd.set_option('display.width', 1000)

def read_benchmarks(tsv_fn, algo=None):
    df = pd.read_csv(tsv_fn, sep='\t', index_col=False)
    df['s_per_pair'] = df['s'] / df['cnt']
    df['s_per_bp'] = df['s'] / (df['cnt'] * df['n'])
    
    if 'align' in df:
        df['align_frac'] = df['align'] / (df['precom'] + df['align'])
        df['prune_frac'] = df['prune'] / df['align']
        df['h_approx_frac'] = df['h0'] / df['ed']
        df['expanded_frac'] = df['expanded'] / df['explored']
    return df

#green palette: #e1dd72, #a8c66c, #1b6535
#blue palette: #408ec6, #7a2048, #1e2761
#colorful: #cf1578, #e8d21d, #039fbe

# adobe:
# analogues blue: #DE4AFF, #625AFF, #4DC8FF
# monochromatic: #FF8746, #805C49, #CC6B37
# monochromatic violet: #F387FF, #AC68FF, #8C8CFF
# mono orange: #FFC545, FF913D, FF6047
# mono red: E8841A, FF6D29, EB2D12

def algo2color(algo):
    palette = sns.color_palette("tab10", 10)
    d = {
        'dijkstra': '#FF913D', #'#E8841A',
        'pa_noprune': '#FF6047',
        'pa': '#EB2D12',
        'pa_manual': 'black',  # (k,m) cherry-picking
        
        'edlib': '#DE4AFF',
        'wfa': '#625AFF',
        'biwfa': '#625AFF',
        
        #'astarix-prefix': '#FF6D29',
        #'astar-prefix': '#FF6D29',
        #'astarix-prefix-illumina': '#FF6D29',
        #'astarix-seeds': '#EB2D12',
        #'astar-seeds': '#EB2D12',
        #'astarix-seeds-illumina': '#EB2D12',
        #'graphaligner': '#8C8CFF',
        #'pasgal': '#AC68FF',
        #'vargas': '#F387FF',
        }
    if algo in d:
        return d[algo]
    assert False, algo
    return '#FF6D29'

def algo2beautiful(algo):
    d = {
        'dijkstra': 'Dijkstra',
        'pa_noprune': 'OSH-noprune',
        'pa': 'OSH',
        'pa_manual': 'OSH-cherrypick',
        
        'edlib': 'Edlib',
        'wfa': 'WFA',
        'biwfa': 'BiWFA',
        }
    if algo in d:
        return d[algo]
    
def algo2marker(algo):
    d = {
        'dijkstra': 'P',
        'pa_noprune': 's',
        'pa': 'o',
        'pa_manual': '.',
        
        'edlib': '^',
        'wfa': 'D',
        'biwfa': 'D',
        }
    if algo in d:
        return d[algo]
    assert False, algo
    #return str(algo)  
    
def col2name(col):
    d = {
        'head':    'Reference size [bp]',
        'head_Mbp':'Reference size [Mbp]',
        's':       'Runtime [s]',
        'n':       'Sequence length [bp]',
        's_per_pair': 'Time per alignment [s]',
        'N':       'Reads',
        'm':       'Read length [bp]',
        'max_rss': 'Memory',
        'score':   'Alignment cost',
        'explored_states':  'Explored states',
        't(map)':  'Alignment time per read',  #  [s/read]
        't(map)_per_bp': 'Alignment time per bp [s]',
        'align_sec':  'Alignment time [s]',
        'cost':    'Alignment cost',
        'explored_per_bp': 'Explored states per bp',
        'error_rate': 'Error rate',
        'bps':     'Alignment rate [bp/s]',
        'spkb':    'Alignment time [s/kbp]',
        #'performance': 'MBp/s'
        }
    if col in d:
        return d[col]
    #print(col)
    return col

def col2unit(col):
    d = {
        'head':    'bp',
        'head_Mbp':'Mbp',
        's':       's',
        'N':       '',
        'm':       'bp',
        'max_rss': 'MB',
        'bps':     'bp/s',
        }
    if col in d:
        return d[col]
    print(col)
    return col

def col2var(col):
    d = {
        'head':    'N',
        'm':       'm',
        }
    if col in d:
        return d[col]
    print(col)
    return col

def readlen2style(readlen):
    d = {
        50:    '.',
        75:    ':',
        100:   '-o',
        150:   '-o',
        }
    if readlen in d:
        return d[readlen]
    print(readlen)
    assert(false)

def readlen2marker(readlen):
    # 'o', 'v', '^', '<', '>', '8', 's', 'p', '*', 'h', 'H', 'D', 'd', 'P', 'X'
    d = {
        75:    '^',
        100:   'o',
        150:   's',
        }
    if readlen in d:
        return d[readlen]
    print(readlen)
    assert(false)
    
def eq(a, b):
    return abs(a-b) < 1e-4

def myticks(num, pos):
        if num == 0: return "$0$"
        exponent = int(np.log10(num))
        coeff = num/10**exponent
        if eq(coeff, 1.0):
            #return r"{:2.0f}".format(num)
            if eq(exponent, 1.0):
                return r"$10$"
            return r"$10^{{ {:2d} }}$".format(exponent)
        assert(False)
        return r"${:2.0f} \times 10^{{ {:2d} }}$".format(coeff,exponent)
