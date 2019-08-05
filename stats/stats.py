import os
import IP2Location
import operator

database = IP2Location.IP2Location("./IP2LOCATION-LITE-DB1.BIN")

stats = {}
for line in file('./result.csv'):
    ip = line.split(':')[0]
    c = database.get_all(ip).country_long
    n = stats.get(c, 0)
    stats[c] = n + 1
sorted_x = sorted(stats.items(), key=operator.itemgetter(1), reverse=True)
for k,v in sorted_x:
    print k, v
