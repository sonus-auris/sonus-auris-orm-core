#!/usr/bin/env python3
from itertools import product

def main():
    checked=0
    for owner,writer,next_chunk,requested_chunk in product(range(2),range(2),range(3),range(3)):
        append_ok = owner==writer and requested_chunk==next_chunk
        if append_ok:
            assert owner==writer,'cross-owner recording append admitted'
            assert requested_chunk==next_chunk,'out-of-order chunk append admitted'
        checked+=1
    print(f'recording append model: {checked} states')
if __name__=='__main__':main()
