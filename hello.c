int half(int x) {
  return x / 2;
}

int f() {
  return 1;
}

int main() {
  return half(10) + f();
}
