if __name__ == "__main__":
    print('[')
    for x in range(3):
        print('[')
        for y in range(3):
            i, j = y, x
            r1, r2 = (i + 1) % 3, (i + 2) % 3
            c1, c2 = (j + 1) % 3, (j + 2) % 3
            print(f'(self[{r1}][{c1}] * self[{r2}][{c2}] - self[{r1}][{c2}] * self[{r2}][{c1}]) / d', end=', ')
        print('],')
    print(']')