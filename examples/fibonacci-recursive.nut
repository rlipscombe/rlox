function fibonacci(n) {
  if (n <= 1) { return n; }
  return fibonacci(n - 2) + fibonacci(n - 1);
}

// Recursive fibonacci, without memoisation, has ridiculously bad performance.
// In fact, O(Fib(n)) ~= Fib(n), if I'm understanding this correctly.
// 20 is a bit too quick, 25 is getting too slow.
for (local i = 0; i < 25; i = i + 1) {
  print(fibonacci(i) + "\n");
}
