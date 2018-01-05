from ctypes import *

rlzjb = cdll.LoadLibrary('/Users/alcari/sauce/rlzjb/target/release/librlzjb.dylib')


class DecompressionResult(Structure):
    _fields_ = [("success", c_byte),
                ("size", c_size_t),
                ("data_ptr", c_void_p),
                ]

    @property
    def data(self):
        x = cast(self.data_ptr, POINTER(c_ubyte*self.size)).contents
        x = bytearray(x)
        return x


rlzjb.decompress_external.argtypes = [c_void_p, c_size_t, c_size_t]
rlzjb.decompress_external.restype = DecompressionResult
rlzjb.free_result.argtypes = [DecompressionResult]


def decompress(bs, size):
    if len(bs) == 0:
        return bytearray()
    ret = rlzjb.decompress_external(bs, len(bs), size)
    if ret.success:
        data = ret.data
        rlzjb.free_result(ret)
        return data
    else:
        raise Exception()


if __name__ == '__main__':
    import sys
    compressed = None
    reference = None
    if len(sys.argv) > 2:
        compressed = sys.argv[1]
        reference = sys.argv[2]
    elif len(sys.argv) > 1:
        target = sys.argv[1]
    else:
        target = '107'
    data = open(compressed or '{}-in'.format(target), 'rb').read()
    res = open(reference or '{}-out'.format(target), 'rb').read()

    print(decompress(data, len(res)) == res)

    l = []
    starts = []
    for x in range(3):
        l.append(decompress(data, len(res)))
        starts.append(l[-1])
    print('done')
    # print(str(res).encode('hex'))
    # print(str(l[0]).encode('hex'))
    # print(str(l[1]).encode('hex'))
    final = [s == res for s in starts]
    # print(final, sum(final))
    assert all(final)
    # raw_input()
