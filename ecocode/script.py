import time

def long_running_task():
    print("Starting long-running task...")
    for _ in range(10000):
        # print("Task is still running...")  # This will continuously print without sleeping
        # Simulate a CPU-heavy task (e.g., calculating something)
        _ = sum(i * i for i in range(10000))  # Just an example of CPU load
        # You can replace the above line with any CPU-intensive operation

if __name__ == "__main__":
    long_running_task()
