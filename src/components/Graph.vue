<template>
  <Line :data="chartData" :options="options" class="graph" />
</template>
     
<script>
import { Line } from 'vue-chartjs'
import { getCssVar } from 'quasar'
import debounce from 'lodash.debounce'
import { useQuasar } from 'quasar'
import {
  Chart as ChartJS,
  LinearScale,
  LogarithmicScale,
  PointElement,
  LineElement
} from 'chart.js'

ChartJS.register(
  LinearScale,
  LogarithmicScale,
  PointElement,
  LineElement
)
import { getFilterCoefficients } from '@/components/FilterCoefficients.js'

const audioCtx = new AudioContext()
const biquadFilter = audioCtx.createBiquadFilter()

const STEPS = 1024;
const frequency = new Float32Array(STEPS)
var magnitudeSum = new Float32Array(STEPS)
var magnitude = []
var phaseResponse = new Float32Array(STEPS)
var previousConfig = []

// We plot with a logarithmic scale, so we copmpensate here to
// get a uniform resolution at either end of the plot.
for (var i = 0; i < STEPS; i++) {
  frequency[i] = Math.pow(20000, i / STEPS);
}

function getTextColor() {
  const $q = useQuasar();
  if ($q.dark !== undefined && $q.dark.isActive) return "rgb(200, 200, 200)"
  return "rgb(140, 140, 140)"
}

export default {
  name: 'LineChart',
  components: { Line },
  watch: {
    filters: {
      handler: debounce(function () {
        magnitudeSum.fill(0);
        if (this.filters !== undefined) {
          const config = this.filters.filters;
          for (var i in config) {
            if (magnitude.length <= i) {
              magnitude.push(new Float32Array(STEPS))
              previousConfig.push(undefined)
            }
            var cfg = JSON.stringify(config[i])
            if (previousConfig[i] !== cfg && config[i].enabled) {
              if (config[i].filter_type == "custom_iir") {
                if (config[i].a0 != 0) {
                  const iirFilter = new IIRFilterNode(audioCtx, { feedforward: [config[i].b0, config[i].b1, config[i].b2], feedback: [config[i].a0, config[i].a1, config[i].a2] })
                  iirFilter.getFrequencyResponse(frequency, magnitude[i], phaseResponse)
                }
              }
              else {
                const iirFilter = new IIRFilterNode(audioCtx, getFilterCoefficients(config[i].filter_type, config[i].f0, config[i].db_gain, config[i].q))
                iirFilter.getFrequencyResponse(frequency, magnitude[i], phaseResponse)
              }
              previousConfig[i] = cfg
            }
            for (var j = 0; j < STEPS; j += 1) {
              if (config[i].enabled) {
                magnitudeSum[j] += magnitude[i][j]
              }
            }
          }
        }
        this.chartData = {
          labels: frequency,
          datasets: [
            {
              label: "title",
              borderColor: getCssVar('primary'),
              data: magnitudeSum,
              stepped: false,
              tension: 0
            }
          ]
        }
      }, 30),
      deep: true
    }
  },
  props: {
    filters: {
      type: Object,
      default: undefined
    }
  },
  data() {
    return {
      chartData: {
        labels: frequency,
        datasets: [
          {
            label: "title",
            borderColor: getCssVar('primary'),
            data: magnitudeSum,
            stepped: false,
            tension: 0
          }
        ]
      },
      options:
      {
        maintainAspectRatio: false,
        animation: true,
        elements:
        {
          point: {
            pointStyle: false
          },
          line: {
            borderWidth: 3
          }
        },
        plugins: {
          legend: {
            display: false
          },
          tooltip: {
            enabled: false
          }
        },
        scales: {
          x: {
            ticks: {
              display: true,
              color: getTextColor()
            },
            grid: {
              display: false
            },
            type: 'logarithmic',
            min: 1,
            max: 20000
          },
          y: {
            ticks: {
              display: false
            },
            grid: {
              display: true
            },
          }
        }
      }
    }
  }
}
</script>