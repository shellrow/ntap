export const setRoutine = ({
    callback, // The callback function
    interval = 5 * 1000, // callback interval
}: {
    callback: Function
    interval: number
}) => {
    let timerId = -1
    // recursive function
    const repeater = () => {
        callback()
        timerId = window.setTimeout(repeater, interval)
    }
    return {
        // start the routine
        start: () => {
        if (timerId !== -1) return
        timerId = window.setTimeout(repeater, interval)
        },
        // stop the routine
        stop: () => {
        if (timerId === -1) return
        clearTimeout(timerId)
        timerId = -1
        },
        // check if the routine is working
        isWorking: () => timerId !== -1
    }
}
