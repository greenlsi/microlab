import { useState } from "react";
import "../styles/TimerView.css";
import { TimerProps } from "../types/fieldTypes";

const TimerView: React.FC<TimerProps> = ({ timerInfo }) => {
    const [showTimerInfo, setShowTimerInfo] = useState(false);
    const [showAllChannels, setShowAllChannels] = useState(false);

    if (!timerInfo) {
        return <div className="timer-container"><p>This pin is not assigned to any Timer.</p></div>;
    }

    const mainChannel = timerInfo.channels.find((ch: any) => ch.channel === timerInfo.channel);

    return (
        <div className="timer-container">
             <p 
                className="toggle-timer-text" 
                onClick={() => setShowTimerInfo(!showTimerInfo)}
            >
                {showTimerInfo ? "Hide Timer Info" : "Show Timer Info"}
            </p>

            {showTimerInfo && (
                 <div className="timer-subcontainer">
                    <h2>{timerInfo.timer}</h2>
                    <p>Selected Channel: {timerInfo.channel}</p>
                    <p>ARR (Auto-Reload Register): {timerInfo.auto_reload}</p>
                    <p>PSC (Prescaler): {timerInfo.prescaler}</p>


                    {mainChannel ? (
                        <div className="main-channel-info">
                            <h3>Channel {mainChannel.channel} Info</h3>
                            <p>Status: {mainChannel.enabled ? "Enabled" : "Disabled"}</p>
                            <p>Mode: {mainChannel.mode}</p>
                            <p>Polarity: {mainChannel.polarity}</p>
                            {mainChannel.duty_cycle !== null && (
                                <p>Duty Cycle: {mainChannel.duty_cycle}%</p>
                            )}
                            {mainChannel.frequency !== null && (
                                <p>Frequency: {mainChannel.frequency} Hz</p>
                            )}
                        </div>
                    ) : (
                        <p>No information found for the selected channel.</p>
                    )}

                    <button 
                        className="show-channels-button" 
                        onClick={() => setShowAllChannels(!showAllChannels)}
                    >
                        {showAllChannels ? "Hide Other Channels" : "Show Other Channels"}
                    </button>

                    {showAllChannels && (
                        <div className="other-channels-info">
                            <h3>Other Channels:</h3>
                            {timerInfo.channels
                                .filter((ch: any) => ch.channel !== timerInfo.channel)
                                .map((ch: any) => (
                                    <div key={ch.channel} className="channel-info">
                                        <h4>Channel {ch.channel}</h4>
                                        <p>Status: {ch.enabled ? "Enabled" : "Disabled"}</p>
                                        <p>Mode: {ch.mode}</p>
                                        <p>Polarity: {ch.polarity}</p>
                                        {ch.duty_cycle !== null && (
                                            <p>Duty Cycle: {ch.duty_cycle}%</p>
                                        )}
                                        {ch.frequency !== null && (
                                            <p>Frequency: {ch.frequency} Hz</p>
                                        )}
                                    </div>
                                ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export default TimerView;