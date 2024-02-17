from typing import List, Optional, Iterator

class Bag:
    def __init__(bag_uri: str):
        """
        Creates a new Bag object from a URI.
        """
        ...

    def read_messages(self, topics: Optional[List[str]] = None, start: Optional[int] = None, end: Optional[int] = None) -> Iterator:
        """
        Reads messages from the bag. Messages are almost guaranteed to be ordered in time.

        Args:
            topics (Optional[List[str]], optional: Topics to include.
                If not specified all topics are included.
                If topic is specified, but does not exists an error is raised.
                Defaults to None (all topics).
            start (Optional[int], optional): Time at which to start reading.
                Defaults to None (start of the bag).
            end (Optional[int], optional): Time at which to stop reading.
                Defaults to None (end of the bag).

        Yields:
            Iterator: Iterator through tuples of:
                1. int - timestamp of message (according to bag, not from header)
                2. int - connection id
                3. MsgValue - deserialized message object
        """
        ...

    def num_messages(self) -> int:
        """_summary_

        Returns:
            int: Number of messages in a bag
        """
        ...