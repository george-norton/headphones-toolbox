import { FilterTypes } from './FilterTypes.js'

const fs = 48000

export function getLowpassCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = (1.0 - cosw0) / 2.0
    var b1 = 1.0 - cosw0
    var b2 = (1.0 - cosw0) / 2.0
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getHighpassCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = (1.0 + cosw0) / 2.0
    var b1 = -(1.0 + cosw0)
    var b2 = (1.0 + cosw0) / 2.0
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getBandpassSkirtCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = sinw0 / 2.0
    var b1 = 0.0
    var b2 = -sinw0 / 2.0
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getBandpassPeakCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = alpha
    var b1 = 0.0
    var b2 = -alpha
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getNotchCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = 1.0
    var b1 = -2.0 * cosw0
    var b2 = 1.0
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getAllPassCoefficients(f0, Q) {
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = 1.0 - alpha
    var b1 = -2.0 * cosw0
    var b2 = 1.0 + alpha
    var a0 = 1.0 + alpha
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - alpha

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getPeakingCoefficients(f0, dBgain, Q) {
    var A = Math.pow(10.0, (dBgain / 40))
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var b0 = 1.0 + (alpha * A)
    var b1 = -2.0 * cosw0
    var b2 = 1.0 - (alpha * A)
    var a0 = 1.0 + (alpha / A)
    var a1 = -2.0 * cosw0
    var a2 = 1.0 - (alpha / A)

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getLowShelfCoefficients(f0, dBgain, Q) {
    var A = Math.pow(10.0, (dBgain / 40))
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var trAa = 2 * Math.sqrt(A) * alpha

    var b0 = A * ((A + 1) - ((A - 1) * cosw0) + trAa)
    var b1 = 2 * A * ((A - 1) - ((A + 1) * cosw0))
    var b2 = A * ((A + 1) - ((A - 1) * cosw0) - trAa)
    var a0 = (A + 1) + ((A - 1) * cosw0) + trAa
    var a1 = -2 * ((A - 1) + ((A + 1) * cosw0))
    var a2 = (A + 1) + ((A - 1) * cosw0) - trAa

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getHighShelfCoefficients(f0, dBgain, Q) {
    var A = Math.pow(10.0, (dBgain / 40))
    var w0 = 2.0 * Math.PI * f0 / fs

    var cosw0 = Math.cos(w0)
    var sinw0 = Math.sin(w0)
    var alpha = sinw0 / (2.0 * Q)

    var trAa = 2 * Math.sqrt(A) * alpha

    var b0 = A * ((A + 1) + ((A - 1) * cosw0) + trAa)
    var b1 = -2 * A * ((A - 1) + ((A + 1) * cosw0))
    var b2 = A * ((A + 1) + ((A - 1) * cosw0) - trAa)
    var a0 = (A + 1) - ((A - 1) * cosw0) + trAa
    var a1 = 2 * ((A - 1) - ((A + 1) * cosw0))
    var a2 = (A + 1) - ((A - 1) * cosw0) - trAa

    return { feedforward: [b0, b1, b2], feedback: [a0, a1, a2] }
}

export function getFilterCoefficients(type, f0, dbGain, q) {
    switch (type) {
        case FilterTypes.LOWPASS:
            return getLowpassCoefficients(f0, q)
        case FilterTypes.HIGHPASS:
            return getBandpassSkirtCoefficients(f0, q)
        case FilterTypes.BANDPASSSKIRT:
            return getBandpassSkirtCoefficients(f0, q)
        case FilterTypes.BANDPASSPEAK:
            return getBandpassPeakCoefficients(f0, q)
        case FilterTypes.NOTCH:
            return getNotchCoefficients(f0, q)
        case FilterTypes.ALLPASS:
            return getAllPassCoefficients(f0, q)
        case FilterTypes.PEAKING:
            return getPeakingCoefficients(f0, dbGain, q)
        case FilterTypes.LOWSHELF:
            return getLowShelfCoefficients(f0, dbGain, q)
        case FilterTypes.HIGHSHELF:
            return getHighShelfCoefficients(f0, dbGain, q)
    }
    return undefined
}