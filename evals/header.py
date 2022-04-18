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

time_limit = 3600  # sec
memory_limit = 20000  # MB

def read_benchmarks_aggregation(benchmarks_file):
    df = pd.read_csv(benchmarks_file, sep='\t', index_col=False)
    df = df.replace(['astarix-seeds-illumina'],'astarix-seeds')
    df = df.replace(['astarix-prefix-illumina'],'astarix-prefix')
    df['algo'] = pd.Categorical(df['algo'], ["astarix-seeds-illumina", "astarix-seeds", "astarix-prefix-illumina", "astarix-prefix", "dijkstra", "graphaligner", "vargas", "pasgal"])
    #df['c'] = df['algo'].apply(algo2color)
    #df['marker'] = df['m'].apply(readlen2marker)
    df['head_Mbp'] = df['head'] / 10**6
    tech = df['sequencing_technology'].iloc[0]
    if 'hifi-natural' == tech:
        head = df['head'].iloc[0] if df['head'].iloc[0] != 100000000 else 5e6 if df['graph'].iloc[0] == 'MHC1' else 1e6 
        coverage = df['N'] * 5e6 / head
        df['bps'] = coverage * head / df['s']
    elif 'illumina' == tech:
        df['bps'] = df['N'] * df['m'] / df['s']
    elif 'hifi' == tech:
        coverage = df['N']
        df['bps'] = coverage * df['head'] / df['s']
    else:
        assert(false)
        
    df['spkb'] = 1e3 / df['bps']
    df['kbps'] = df['bps'] * 1e-3
    df['GB'] = df['max_rss'] * 1e-3
    
    df = df.sort_values(by='algo').reset_index(drop=True)
    return df

def remove_tle_mle(df):
    df = df.loc[df.s <= time_limit]
    df = df.loc[df.max_rss <= memory_limit]
    return df

def num_lower(serie):
    return serie.apply(lambda s: sum(1 for c in s if c.islower()))

def read_benchmarks(tsv_fn, algo=None):
    df = pd.read_csv(tsv_fn, sep='\t', index_col=False)
    df['s_per_pair'] = df['s'] / df['cnt']
    
    if 'align' in df:
        df['align_frac'] = df['align'] / (df['precom'] + df['align'])
        df['prune_frac'] = df['prune'] / df['align']
        df['h_approx_frac'] = df['h0'] / df['ed']
        df['expanded_frac'] = df['expanded'] / df['explored']
    #df = df.groupby(["alg", "cnt", "e"]).median()

    #df['pushed+popped'] = df['pushed'] + df['popped']
    #df['explored_per_bp'] = df['explored_states'] / df['len']
    #df['t(map)_per_bp'] = df['t(map)'] / df['len']
    #if 'crumbs' in df:
    #    df['crumbs_per_bp'] = df['crumbs'] / df['len']
    #df['error_rate'] = df['cost'] / df['len']
    #df['generated_errors'] = df['readname'].apply(lambda rn: int(rn.split()[0]) if rn.split()[0].isdigit() else -1)  # TODO: uncomment
    #df['explored_states'] = df['pushed'] * df['len']
    #df['algo'] = df['algo'].replace(['astar-prefix'], 'astarix')
    
    #if algo:
    #    df['algo'] = algo
    #else:
    #    df['algo'] = pd.Categorical(df['algo'], ["astarix-seeds", "astar-seeds", "astarix-seeds-illumina", "graphaligner", "dijkstra", "astar-prefix", "pasgal"], ordered=True)
    
    #df['algo'] = df['algo'].cat.remove_unused_categories()
    #df['performance'] = df['len'] / df['t(map)'] / 1000000  # [MBp/sec]
    #if 'spell' in df:  # TODO: uncomment
    #    df['dist'] = num_lower(df['spell'])
    #return df.set_index('readname', verify_integrity=True)
    #return df.set_index('readname', verify_integrity=False)
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
        'dijkstra': '#E8841A',
        'pa_noprune': '#FF6D29',
        'pa_inf': '#EB2D12',
        'pa': 'black',  # (k,m) cherry-picking
        
        'edlib': '#DE4AFF',
        'biwfa': '#625AFF',
        'astarix-prefix': '#FF6D29',
        'astar-prefix': '#FF6D29',
        'astarix-prefix-illumina': '#FF6D29',
        'astarix-seeds': '#EB2D12',
        'astar-seeds': '#EB2D12',
        'astarix-seeds-illumina': '#EB2D12',
        'graphaligner': '#8C8CFF',
        'pasgal': '#AC68FF',
        'vargas': '#F387FF',
        }
    if algo in d:
        return d[algo]
    assert False, algo
    return '#FF6D29'

def algo2beautiful(algo):
    d = {
        'dijkstra': 'Dijkstra',
        'pa_noprune': 'OSH-noprune',
        'pa_inf': 'OSH',
        'pa': 'OSH-cherrypick',
        
        'edlib': 'Edlib',
        'wfa': 'WFA',
        'biwfa': 'BiWFA',
        }
    if algo in d:
        return d[algo]
    return str(algo)
    
def col2name(col):
    d = {
        'head':    'Reference size [bp]',
        'head_Mbp':'Reference size [Mbp]',
        's':       'Runtime [s]',
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
