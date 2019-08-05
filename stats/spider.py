#coding=utf-8
import requests, time, re
urls = []
date = []
for i in range(1, 6):
    for j in range(2, 26, 4):
        i_ = str(i)
        if i < 10:
            i_ = '0%i'%i
        if j < 10:
            j_ = '0%i_00'%j
        else:
            j_ = '%i_00'%j
        url = 'http://grin-map.cycle42.com/data/2019/Aug/%s/%s/'%(i_, j_)
        urls.append(url)

        date.append('2019-08-%s %s'%(i, j))
        #print url

for i, u in enumerate(urls):
    c = requests.get(u).content
    if c.find('404') != -1:
        continue
    
    total_count = re.findall('Total nodes:\s*(\d+)\s*', c)[0]
    public_count = re.findall('Public nodes:\s*(\d+)\s*', c)[0]
    print date[i], total_count, public_count 
    time.sleep(0.5)
