import torch

def long_running_task():
    for _ in range(100000):
        tensor = torch.arange(1000000, device='cuda')  # Create tensor on GPU
        result = torch.sum(tensor * tensor)  # GPU computation

if __name__ == "__main__":
 
    long_running_task()
