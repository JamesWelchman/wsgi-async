
from wsgi_async_core import RequestThread


REQUEST_THREAD = RequestThread()


__all__ = [
    "Future",
]


class RequestFuture:
    def __init__(self, method, url, headers=None, payload=b"", timeout=None):
        headers = headers or {}
        self.timeout = timeout
        self._ref = REQUEST_THREAD.send(url, headers, payload)
        self._complete = False

    def is_complete(self):
        return self._complete

    def wait(self):
        try:
            if self.is_complete():
                raise RuntimeError("response already taken")

            return REQUEST_THREAD.wait(self._ref)
        finally:
            self._complete = True

    def try_take(self):
        if self.is_complete():
            raise RuntimeError("response already taken")

        res = REQUEST_THREAD.try_take(self._ref)
        if res:
            self._complete = True

        return res
