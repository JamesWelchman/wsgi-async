
from wsgi_async import RequestFuture


def application(environ, start_response):
    start_response("200 OK", [])
    # Do 15 requets in parallel
    futures = []
    for _ in range(15):
        fut = RequestFuture("GET",
                            "https://google.com",
                            headers=None,
                            payload=b"")
        futures.append(fut)

    # wait for all fifteen requests to complete
    resps = []
    for f in futures:
        resps.append(f.wait())

    return (b"",)
