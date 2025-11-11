import { useMemo, useState } from "react";

import defaultConfig from "../.kprivate/config.txt?raw";
import { main } from "./ncs";
import { Input, Textarea } from "@mantine/core";

const small = `
interface HundredGigE0/0/0/0.100 l2transport
 encapsulation dot1q 300
 rewrite ingress tag pop 1 symmetric
!
l2vpn
 bridge group VLAN
  bridge-domain VLAN100
   interface HundredGigE0/0/0/0.100
   !
`;

function App() {
  const [src, setSrc] = useState(defaultConfig);
  const currentConfig = useMemo(() => {
    return main(src);
  }, [src]);

  return (
    <>
      <pre>
        <code style={{ whiteSpace: "pre" }}>{currentConfig.lint()}</code>
      </pre>

      <h3>config</h3>
      <Textarea
        style={{ fontFamily: "monospace" }}
        onChange={(e) => {
          setSrc(e.target.value);
        }}
        value={src}
      />
    </>
  );
}

export default App;
