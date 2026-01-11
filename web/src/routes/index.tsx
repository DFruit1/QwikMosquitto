import { component$ } from "@builder.io/qwik";

export default component$(() => {
  return (
    <main>
      <h1>Qwik Mosquitto Recorder</h1>
      <p>
        This UI will surface MQTT messages and system status once the API layer is
        wired in.
      </p>
    </main>
  );
});
