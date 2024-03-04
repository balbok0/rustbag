from typing import Dict, List, Optional, Iterator

class Bag:
    def __init__(bag_uri: str, storage_options: Optional[Dict[str, str]] = None):
        """
        Creates a new Bag object from a URI.

        Args:
            bag_uri (str): A URI string pointing to a bag, should start with either:
                - "file://"
                - "http://"
                - "https://"
            storage_options (Optional[Dict[str, str]], optional): Storage options to use when reading URI.
                For allowed keys/values see [object_store docs](https://docs.rs/object_store/0.9.0/object_store/aws/enum.AmazonS3ConfigKey.html)
                (or similar page for non-S3 storage).
                Defaults to None, i.e. default object store configuration is used.
        """
        ...

    def read_messages(self, topics: Optional[List[str]] = None, start: Optional[int] = None, end: Optional[int] = None, config: Optional[Dict[str, str]] = None) -> Iterator:
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
            config (Optional[Dict[str, str]]): Configuration of the reader.
                Currently allowed keys are: "num_threads".
                Defaults to None (Default configuration).

        Yields:
            Iterator: Iterator through tuples of:

                1. int - timestamp of message (according to bag, not from header)
                2. int - connection id
                3. MsgValue - deserialized message object
        """
        ...

    def num_messages(self) -> int:
        """
        Returns:
            int: Number of messages in a bag
        """
        ...