import asyncio
import websockets
import json
import subprocess
import uuid

async def communicate_with_server():
    uri = "ws://localhost:8080"
    async with websockets.connect(uri) as websocket:
        # تولید UUID یکتا
        call_uuid = str(uuid.uuid4())

        function_call = {
            "function": "add",
            "uuid": call_uuid,
            "args": {"a": 10, "b": 4},
            "command": "python3",
            "location": "../src/add.py"
        }

        # ارسال پیام به سرور
        await websocket.send(json.dumps(function_call))
        print(f"Sent message: {function_call}")

        # انتظار برای دریافت پاسخ از سرور
        while True:
            response = await websocket.recv()
            print(response)
            response_data = json.loads(response)
            if response_data['uuid'] == call_uuid:
                print(f"Received response: {response_data['result']}")
                break

        # دریافت پیام از سرور و پردازش آن‌ها
        while True:
            message = await websocket.recv()
            data = json.loads(message)
            command = data.get('command')
            location = data.get('location')
            args = data.get('args')

            if command and location and args:
                process = subprocess.Popen(
                    [command, location],
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE
                )

                input_data = json.dumps(args)
                stdout, stderr = process.communicate(input_data.encode('utf-8'))
                result_data = json.loads(stdout.decode('utf-8'))

                response = {
                    "uuid": data['uuid'],
                    "result": result_data['result']
                }

                await websocket.send(json.dumps(response))
                print(f"Sent response: {response}")

if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(communicate_with_server())
