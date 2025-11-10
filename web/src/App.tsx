import { useEffect, useMemo, useState } from "react";
import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";

import defaultConfig from "../.kprivate/config.txt?raw";
import { main } from "./ncs";

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
      <pre>
        <code style={{ whiteSpace: "pre" }}>{src}</code>
      </pre>
    </>
  );
}

export default App;
