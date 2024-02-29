# `Pin` Explanation

Pin ensures that the Future is "pinned" in place and won't be moved in memory, allowing us to safely hold these internal or "self-referential" pointers. So, Pin's role is about ensuring safety and soundness when dealing with these Futures or any self-referential structures, in an asynchronous context.
