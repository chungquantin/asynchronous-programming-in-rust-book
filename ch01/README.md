# Chapter 1 - Concurrency and Asynchronous Programming: A Detailed Overview

This folder contains the code examples for Chapter 1.

# CHAPTER NOTES

## Async history

Back to the day, computer is designed to only run one single task at a time => punchcard machine when there is no OS, no thread, no scheduling, no multitasking. Later, DOS was invented and OS manages the processes in a better way with scheduler, resource sharing...

### Non-preemptive multitasking (cooperative multitasking)

In non-preemptive multitasking, the CPU control largely remains with one program for longer durations. Non-preemptive multitasking works well with applications and programs that require intensive and continuous CPU resources. However, when a program holds the CPU for such long periods, it affects other programs that must wait for the current program to finish or voluntarily release the CPU.

Non-preemptive multitasking also incorporates some elements from cooperative multitasking, where one or more programs cooperate, to some extent, in CPU utilization sharing and collaboration.

**Use case**: Interactive user interface, background processes (child forking).

### Preemptive multitasking (time-shared multitasking)

In preemptive multitasking, computer programs share operating system and underlying hardware resources. It divides the overall operating and computing time between processes, and the switching of resources between different processes occurs through predefined criteria.

### Hyper-threading

As CPUs evolved and added more functionality such as several **arithmetic logic units (ALUs)** and **additional logic units**, the CPU manufacturers realized that the entire CPU wasn't fully utilized. For example, when an operation only required some parts of the CPU, an instruction could be run on the ALU simultaneously. This became the start of hyper-threading.

### Multicore processors

Processor today is designed to be very small which allows the CPU to contain multiple processors on the integrated circui leading to the born of multicore CPUs.

> Each core nowaday has the ability to perform hyper-threading

### NOTES

- Out-of-order execution: Processor decides the order of the instructions which will be executed next if it believes doing that way makes the computer faster. So A can happens before B => `race condition if there is mutex lock on resource shared`

- Modern CPUs offload some works to separate coprocessors such as FPU (floating point calculations)

## Concurrency and parallelism

- **Parallelism** is increasing the resources we use to solve a task. It has nothing to do with efficiency.
- **Concurrency** has everything to do with efficiency and resource utilization. Concurrency can never make one single task go faster. It can only help us utilize our resources better and thereby finish a set of tasks faster.

Example:

If we have `8 tasks` => Each task takes 1 hour to finish:

- Parrallelism: Two threads do 4 tasks => Takes `4 / 2 hours = 2 hours` with 2 threads to finish 4 tasks => `4 hours` in total to finish 8 tasks.
  > Faster to finish a set of tasks with shorter time taken, more resources. Can't stop => not interuptible.
- Concurrency: Each thread do 4 tasks => Takes `4 hours` => Still `4 hours` in total. But the resources are shared between threads.
  > Don't focus resources into one set of tasks but share resources across thread workers. Hence, more efficiency in a long run. Can stop and resume => interuptible

### Bartender Exercises

#### Requirements

- Pour the Guinness draught into a glass tilted at 45 degrees until it’s 3-quarters full (`15 seconds`).
- Allow the surge to settle. (`100 seconds`)
- Fill the glass completely to the top (`5 seconds`).
- Serve.

If we do all task in a synchronous way => `120 seconds` = `2 minutes` / `one customer`

#### Alternative 1: Fully synchronous task execution with one bartendar

- Number of customers: 360
- Number of bartender: 1
- Time per one customer: 2 minutes

=> Total: `720 minutes`

> Too slow

#### Alternative 2: Parallel and synchronous task execution

- Number of customers: 360
- Number of bartender: 12
- Time per one customer: 2 minutes / 12

=> Total: `360 * 2 / 12` = `60 minutes`

> Faster but more resources (more bartenders)

#### Alternative 3: Asynchronous task execution with one bartender

While waiting for other tasks to finish, the bartender can work on the another task. But there is a case that while they are doing on one task, the another task is already finished for a while.

=> The order of execution extremely matters in this case if we want high throughput (like calculating the priority point of the order of task and pick the one which should be executed first instead of choosing the random one, hence, we can optimize the execution).

#### Alternative 4: Parallel and synchronous task execution with two bartender

Allows other bartender to steal the task from the another if the another bartender is busy doing other things. Hence, the throughput will be higher.

> Concurrency is about working smarter while parallelism is about throwing more resources at the problem

### Definition of asynchronous programming

Asynchronous programming is the way a programming language or library abstracts over concurrent operations, and how we as users of a language or library use that abstraction to execute tasks concurrently.

## The operating system and the CPU

Operating system participates in any parts of programming, unless you work in the embedded environment or building an operating system. Software relies primarily on the OS.

CPU processes work in a preemptive multitasking way. Hence, your instructions to the CPU won't be executed instruction by instruction but there will be interruption depends on the scheduling mechanism in the CPU design.

- `syscall`: a general term for communication methods between the kernel and programs on the userland
- `libc`: alternative approaches to the `syscall` in the UNIX family of kernels.
- `posix threads`: threads in the UNIX-like OS
- `WinAPI`: syscall in Windows operating system
  > Read more: epoll, kqueue, and IOCP work

## Interrupts, firmware, and I/O”

### How async code is executed under the hood?

1. Code invoke `syscall` methods (`read()`)
2. Sending a signal to the kernel
3. Register a new event with the OS (`Read`)

### Interrupts

### Hardware Interrupts

Hardware interrupts are created by sending an electrical signal through an IRQ. These hardware lines signal the CPU directly.

### Software Interrupts

These are interrupts issued from software instead of hardware. As in the case of a hardware interrupt, the CPU jumps to the IDT and runs the handler for the specified interrupt.
