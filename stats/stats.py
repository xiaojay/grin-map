import os
import IP2Location
import operator

database = IP2Location.IP2Location("./IP2LOCATION-LITE-DB1.BIN")

stats = {}
stats2 = {}
public_count = 0
for line in file('./result.csv'):
    ip = line.split(':')[0]
    c = database.get_all(ip).country_long
    n = stats.get(c, 0)
    stats[c] = n + 1
    if line.split(',')[-1].find('[]') == -1:
        public_count += 1
        n2 = stats2.get(c, 0)
        stats2[c] = n2 + 1

sorted_x = sorted(stats.items(), key=operator.itemgetter(1), reverse=True)
sorted_x2 = sorted(stats2.items(), key=operator.itemgetter(1), reverse=True)
for k,v in sorted_x:
    print k, v

print "=" * 30
print 'PULBIC'
print public_count
for k,v in sorted_x2:
    print k, v

