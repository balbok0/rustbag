from pathlib import Path
import tqdm

import rosbag
from rosbags.highlevel import AnyReader
from rosbags_rs import Bag
from embag import View

path = Path(__file__).parent.parent / "HMB_1.bag"

# create reader instance and open for reading
with AnyReader([path]) as reader:
    for connection, timestamp, rawdata in tqdm.tqdm(reader.messages(), desc="rosbags"):
        msg = reader.deserialize(rawdata, connection.msgtype)
        # d = msg.fields()

bag = Bag(str(path))
for msg in tqdm.tqdm(bag.read_messages(), desc="rs"):
    pass

view = View()
view.addBag(str(path))

for msg in tqdm.tqdm(view.getMessages(), desc="embag"):
    deser_data = msg.data()
    pass

bag = rosbag.Bag(str(path))
for msg in tqdm.tqdm(bag.read_messages(), desc="Default rosbag"):
    pass