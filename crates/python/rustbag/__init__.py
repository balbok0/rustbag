try:
    from .rustbag import *
except ImportError:
    # NOTE: Hack so that docs job can succeed without a problem
    pass

__all__ = ["Bag"]