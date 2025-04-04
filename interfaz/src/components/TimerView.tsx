//Devuelve si es un timer:
import { timer } from "d3";
import "../styles/TimerView.css";
import { TimerProps } from "../types/fieldTypes";

const TimerView: React.FC<TimerProps> = ({ timerInfo }) => {


		return (
				<div className="timer-container">
						{timerInfo ? (
								<>
										<h2>{timerInfo.timer}</h2>
										<p>Channel: {timerInfo.channel}</p>
										<p>ARR: {timerInfo.ARR}</p>
										<p>PSC: {timerInfo.PSC}</p>
										{timerInfo.DC !== undefined && <p>Duty cycle: {timerInfo.DC}</p>}
								</>
						) : (
								<p>Este pin no tiene un Timer asignado</p>
						)}
				</div>
		);
};

export default TimerView;
