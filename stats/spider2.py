#coding=utf-8
import requests, time, re
urls = []
date = []
days = []
for i in range(25, 32):
    for j in range(2, 26, 4):
        i_ = str(i)
        if i < 10:
            i_ = '0%i'%i
        if j < 10:
            j_ = '0%i_00'%j
        else:
            j_ = '%i_00'%j
        url = 'http://grin-map.cycle42.com/data/2019/Jul/%s/%s/'%(i_, j_)
        urls.append(url)

        date.append('2019-07-%s %s'%(i, j))
        days.append(i)
        #print url

y = days[0]
s1 = 0 
s2 = 0
count = 0
for i, u in enumerate(urls):
    c = requests.get(u).content
    if c.find('404') != -1:
        continue
    try:
        total_count = re.findall('Total nodes:\s*(\d+)\s*', c)[0]
        public_count = re.findall('Public nodes:\s*(\d+)\s*', c)[0]
    except IndexError:
        print u
        print 'failed to parse'
        continue
    #print date[i], total_count, public_count 
    t = days[i]
    if t == y:
        s1 += int(total_count)
        s2 += int(public_count)
        count += 1
    else:
        print y, int(s1*1.0/count), int(s2*1.0/count)
        s1 = int(total_count)
        s2 = int(public_count)
        count = 1
        y = t    
    time.sleep(0.25)

print y, int(s1*1.0/count), int(s2*1.0/count)
