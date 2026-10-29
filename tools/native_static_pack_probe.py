# Bounded local protocol probe; packet IDs/layouts read from this checkout.
import socket,struct,uuid,zlib,json
import argparse
parser=argparse.ArgumentParser(description='Check static-content refusal before play on a local scratch Horizon server')
parser.add_argument('--port',type=int,required=True)
port=parser.parse_args().port
assert 1<=port<=65535
def vi(n):
 out=bytearray()
 while True:
  b=n&127; n>>=7; out.append(b|(128 if n else 0))
  if not n:return bytes(out)
def string(s):
 b=s.encode();return vi(len(b))+b
def readvi(buf,offset=0):
 n=0
 for i in range(5):
  b=buf[offset+i];n|=(b&127)<<(7*i)
  if b<128:return n,offset+i+1
 raise ValueError('long varint')
results=[]
for case,name in [('absent','NoNativePack'),('different','WrongNativePack')]:
 with socket.create_connection(('127.0.0.1',port),timeout=15) as sock:
  sock.settimeout(15); compression=None
  def exact(n):
   out=bytearray()
   while len(out)<n:
    b=sock.recv(n-len(out))
    if not b: raise EOFError('connection closed before expected refusal')
    out.extend(b)
   return bytes(out)
  def send(body):
   if compression is not None:
    body=vi(len(body))+zlib.compress(body) if len(body)>=compression else b'\0'+body
   sock.sendall(vi(len(body))+body)
  def recv():
   size=0
   for i in range(5):
    b=exact(1)[0];size|=(b&127)<<(7*i)
    if b<128:break
   body=exact(size)
   if compression is not None:
    unpacked,at=readvi(body);body=zlib.decompress(body[at:]) if unpacked else body[at:]
   ident,at=readvi(body);return ident,body[at:]
  send(vi(0)+vi(777)+string('localhost')+struct.pack('>H',port)+vi(2))
  send(vi(0)+string(name)+uuid.uuid3(uuid.NAMESPACE_DNS,name).bytes)
  for _ in range(10):
   ident,body=recv()
   if ident==3:compression,_=readvi(body)
   elif ident==2:send(vi(3));break
   else:raise RuntimeError(('unexpected login packet',ident,body[:100]))
  else:raise RuntimeError('login did not finish')
  for _ in range(10):
   ident,body=recv()
   if ident==15:
    assert b'static-content-v1' in body
    selected=vi(0) if case=='absent' else vi(1)+string('horizon')+string('static-content-v1')+string('0'*64)
    send(vi(7)+selected)
   elif ident==2:
    assert b'Static content mismatch' in body,body
    results.append({'case':case,'result':'PASS','refused_before_play':True});break
   elif ident==3:raise RuntimeError('server incorrectly entered play')
  else:raise RuntimeError('missing configuration refusal')
print(json.dumps(results))
