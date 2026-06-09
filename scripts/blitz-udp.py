import socket

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# SET player1.pos = 10,20
packet = bytes([0x02, 11]) + b"player1:pos" + bytes([5]) + b"10,20"
s.sendto(packet, ("127.0.0.1", 6380))
print(s.recvfrom(256))

# GET player1.pos
packet = bytes([0x01, 11]) + b"player1:pos"
s.sendto(packet, ("127.0.0.1", 6380))
print(s.recvfrom(256))
