import usb
import sys

dev = usb.core.find(idVendor=0xC0DE, idProduct=0xCAFE)
# was it found?
if dev is None:
    raise ValueError("Device not found")

print(dev)

# Send vendor control request (0x40) to interface (0x1)
dev.ctrl_transfer(0x41, 100, 200)

# get an endpoint instance
cfg = dev.get_active_configuration()

intf = cfg[(0, 0)]

ep_out = intf.endpoints()[0]
ep_in = intf.endpoints()[1]

ep_out.write(sys.argv[1])

print(bytes(ep_in.read(64)).decode())
