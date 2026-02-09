import pandas as pd

# Load CSVs
df1 = pd.read_csv('pytorch_energy.csv')
df2 = pd.read_csv('pytorch_energy2.csv')

# Convert timestamps to datetime
df1['timestamp'] = pd.to_datetime(df1['timestamp'])
df2['timestamp'] = pd.to_datetime(df2['timestamp'])

# Round to nearest second for joining (since readings differ by ~5ms)
df1['timestamp_sec'] = df1['timestamp'].dt.floor('s')
df2['timestamp_sec'] = df2['timestamp'].dt.floor('s')

# Merge on timestamp
merged = pd.merge(df1, df2, on='timestamp_sec', suffixes=('_p1', '_p2'))

# Compare GPU power - total and per-process
print(merged[[
    'timestamp_sec', 
    'gpu_power_watts_p1', 
    'gpu_power_watts_p2',
    'process_gpu_power_watts_p1',
    'process_gpu_power_watts_p2'
]])

# Sum of process GPU powers should be close to total GPU power
merged['sum_process_gpu'] = merged['process_gpu_power_watts_p1'] + merged['process_gpu_power_watts_p2']
print("\n--- Validation: Sum of process GPU vs Total GPU ---")
print(merged[['timestamp_sec', 'gpu_power_watts_p1', 'sum_process_gpu']])