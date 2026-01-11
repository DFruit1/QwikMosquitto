import { renderToString, type RenderToStringOptions } from "@builder.io/qwik/server";
import Root from "./root";

export default function (options: RenderToStringOptions) {
  return renderToString(<Root />, options);
}
