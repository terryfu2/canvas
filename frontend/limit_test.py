import websocket
import json
import threading
import time
import math

# Function to connect to the proxy server and start placing pixels


def worker(worker_id, num_workers, start_time):
    # Connect to the WebSocket server
    ws = websocket.WebSocket()
    ws.connect("ws://localhost:3001/ws")

    square_size = math.floor(math.sqrt(num_workers))
    x = worker_id % square_size
    y = math.floor(worker_id / square_size)

    # Function to calculate gradient color
    def calculate_gradient_color():
        red_value = int((num_workers - worker_id) * 255 / num_workers)
        blue_value = int(worker_id * 255 / num_workers)
        return red_value * 256 * 256 + blue_value

    def generate_color():
        percentage = worker_id / num_workers
        yellow_value = 255
        green_value = int(percentage * 255)
        color = yellow_value * 256 * 256 + green_value * 256
        return color

    # Function to calculate cycle color
    def calculate_cycle_color(cycle_count):
        if cycle_count % 2 == 1:
            return generate_color()
        else:
            return calculate_gradient_color()

    # Function to place pixels
    def place_pixels():
        cycle_count = 0

        # Calculate delay time based on current time
        current_time = time.time()
        elapsed_time = current_time - start_time
        start_delay = 9 / num_workers * worker_id - elapsed_time
        if start_delay > 0:
            time.sleep(start_delay)

        while True:
            color = calculate_cycle_color(cycle_count)
            cycle_count += 1

            # Send message to set pixel
            print(f"{worker_id}: {{{x}, {y}}}, {color}")
            message = {
                "command": "set_pixel",
                "payload": {
                    "x": x,
                    "y": y,
                    "colour": color
                }
            }
            ws.send(json.dumps(message))
            time.sleep(9)

    # Start placing pixels in a separate thread
    threading.Thread(target=place_pixels, daemon=True).start()


# Create 10 workers
num_workers = int(math.pow(10, 2))
start_time = time.time()
for i in range(num_workers):
    threading.Thread(target=worker, args=(
        i, num_workers, start_time), daemon=True).start()

# Keep the main thread alive
while True:
    time.sleep(1)
