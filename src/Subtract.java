import org.json.JSONObject;
import java.nio.charset.StandardCharsets;
import jnr.ffi.LibraryLoader;
import jnr.ffi.Pointer;
import jnr.ffi.Runtime;
import jnr.ffi.annotations.Out;
import jnr.ffi.byref.PointerByReference;

public class Subtract {

    // Define the Rust core library interface
    public interface RustCore {
        Pointer read_from_input_queue();
        int remove_from_input_queue(String uuid);
        int write_to_output_queue(String data);
    }

    private static final RustCore rustCore;

    static {
        rustCore = LibraryLoader.create(RustCore.class).load("./librust_core.so");
    }

    // Function to process the message
    public static void processMessage(String message) {
        try {
            // Parse the JSON message
            JSONObject json = new JSONObject(message);
            int minuend = json.getJSONObject("args").getInt("minuend");
            int subtrahend = json.getJSONObject("args").getInt("subtrahend");
            String uuid = json.getString("uuid");

            // Perform the subtraction
            int result = minuend - subtrahend;

            // Construct response message
            JSONObject response = new JSONObject();
            response.put("uuid", uuid);
            response.put("result", result);

            // Write the response to the output queue
            rustCore.write_to_output_queue(response.toString());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public static void main(String[] args) {
        long startTime = System.currentTimeMillis();
        long timeout = 10000; // 10 seconds timeout

        while (System.currentTimeMillis() - startTime < timeout) {
            // Read message from input queue
            Pointer messagePointer = rustCore.read_from_input_queue();
            if (messagePointer != null) {
                String message = messagePointer.getString(0, StandardCharsets.UTF_8);

                // Check if the message is for the subtract function
                try {
                    JSONObject json = new JSONObject(message);
                    String function = json.getString("function");

                    if ("subtract".equals(function)) {
                        // Process the message
                        processMessage(message);

                        // Remove message from input queue
                        String uuid = json.getString("uuid");
                        rustCore.remove_from_input_queue(uuid);

                        // Exit the loop after processing the message
                        break;
                    }
                } catch (Exception e) {
                    e.printStackTrace();
                }
            }

            // Wait a bit before reading the next message
            try {
                Thread.sleep(100);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                break;
            }
        }

        System.out.println("Timeout reached or message processed.");
    }
}