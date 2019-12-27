# $ python quadratic-problem.py -x `hone param x float -- -100 100` -y `hone param y choice -- -1 0 1`
import argparse

import sklearn.datasets
import sklearn.ensemble
import sklearn.model_selection
import sklearn.svm

parser = argparse.ArgumentParser()
parser.add_argument('-x', type=float, default=0)
parser.add_argument('-y', type=float, default=0)
args = parser.parse_args()

x = args.x
y = args.y

print(x**2 + y)
