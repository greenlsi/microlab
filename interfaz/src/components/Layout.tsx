import { ReactNode } from "react";
import "../styles/Layout.css";

const Layout = ({ children }: { children: ReactNode }) => {
  return (
    <div className="layout">
      <header className="header">
        <h1>MICROLAB - Control Board</h1>
      </header>
      <main className="main-content">{children}</main>
      <footer className="footer">
					<div className="links">
							<a href="https://www.st.com/resource/en/reference_manual/dm00135183-stm32f446xx-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf" target="_blank" rel="noreferrer">Reference Manual</a>
							<a href="https://www.st.com/resource/en/datasheet/stm32f446re.pdf" target="_blank" rel="noreferrer">Datasheet</a>
							<a href="https://www.st.com/resource/en/user_manual/um1724-stm32-nucleo64-boards-mb1136-stmicroelectronics.pdf" target="_blank" rel="noreferrer">User Manual</a>
					</div>
					<div className="author">
							<p>© 2025 - MICROLAB - ETSIT UPM</p>
					</div>
			</footer>
    </div>
  );
};

export default Layout;
