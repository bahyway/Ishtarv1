import json,sys
kg=json.load(open(sys.argv[1]))
out=open(sys.argv[2],'w')
out.write('# ANSHARV4 lines · generated from bahyway_kg.json · PB-646\n')
for n in kg['nodes']:
    out.write('NODE %s|%s|%s|%s|%s\n'%(n['id'],n['type'],
      n['label'].replace('|','/'),n.get('clearance','PUBLIC'),
      ''.join(n.get('pillars',[]))))
for e in kg['edges']:
    out.write('EDGE %s|%s|%s|%s\n'%(e['from'],e['to'],e['rel'],e['line']))
for h in kg.get('hyperedges',[]):
    out.write('HYPER %s|%s|%s|%s\n'%(h['id'],h['kind'],
      h['label'].replace('|','/'),','.join(h['members'])))
out.close()
print('anshar lines written:',sys.argv[2])
