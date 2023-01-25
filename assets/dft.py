#!/usr/bin/env python3

# We use Python to translate picture interpolation points (given as complex numbers in JSON file)
#  into Fourier coefficients (also complex numbers) using Discrete Fourier Transform (DFT).

import json
import math


class Complex:
    def __init__(self, real, imag):
        self.real = real
        self.imag = imag

    def __add__(self, other):
        return Complex(self.real + other.real, self.imag + other.imag)

    def __sub__(self, other):
        return Complex(self.real - other.real, self.imag - other.imag)

    def __mul__(self, other):
        return Complex(self.real * other.real - self.imag * other.imag, self.real * other.imag + self.imag * other.real)

    def __abs__(self):
        return (self.real * self.real + self.imag * self.imag) ** 0.5


def dft(points):
    X = []
    N = len(points)
    for k in range(N):
        if k % 100 == 0:
            print(f"Progress: {k}/{N}")
        res = Complex(0, 0)
        for n in range(N):
            phi = (2 * math.pi * k * n) / N
            c = Complex(math.cos(phi), -math.sin(phi))
            res = res + points[n] * c
        res.real = res.real / N
        res.imag = res.imag / N
        freq = k
        amp = abs(res)
        phase = math.atan2(res.imag, res.real)
        X.append({"re": res.real, "im": res.imag, "freq": freq, "amp": amp, "phase": phase})
    return X


if __name__ == '__main__':
    # 🫢 Coincidence of names
    with open('andrzej.json') as f:
        json_obj = json.load(f)
        data = json_obj['points']
        metadata = json_obj['metadata']

    data = list(map(lambda x: Complex(x['re'], x['im']), data))
    # We take every 10th point to speed up the process.
    # With all points it is really slow to render the image with EGUI.
    data = data[::10]
    dft_data = dft(data)

    with open('dft_andrzej.json', 'w') as f:
        json.dump({"epicycles": dft_data, "metadata": metadata}, f)
