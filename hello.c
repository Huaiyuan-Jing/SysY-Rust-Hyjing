int times2(int x)
{
  return x * 2;
}

int f()
{
  return 1;
}

int main()
{
  return times2(f()) + f();
}
