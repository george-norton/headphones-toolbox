<template>
  <Line :data="chartData" :options="options" class="graph"/>
</template>
     
<script>
import { Line } from 'vue-chartjs'
import { ref, reactive } from 'vue'
import { getCssVar } from 'quasar'
import debounce from 'lodash.debounce'
import { useQuasar } from 'quasar'
import {
  Chart as ChartJS,
  CategoryScale,
  LogarithmicScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
} from 'chart.js'

ChartJS.register(
  CategoryScale,
  LinearScale,
  LogarithmicScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend
)

const audioCtx = new AudioContext()
const biquadFilter = audioCtx.createBiquadFilter()

const STEPS = 1024;
const frequency = new Float32Array(STEPS);
var magnitudeSum = new Float32Array(STEPS);
var magnitude = new Float32Array(STEPS);
var phaseResponse = new Float32Array(STEPS);

// We plot with a logarithmic scale, so we copmpensate here to
// get a uniform resolution at either end of the plot.
for (var i = 0; i < STEPS; i++) {
  frequency[i] = Math.pow(20000, i/STEPS);
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
        if (this.filters == undefined)
          return;
        magnitudeSum.fill(0);
        const config = this.filters.filters;
        for (var i in config) {
          if (config[i].enabled) {
            biquadFilter.type = config[i].filter_type;
            biquadFilter.frequency.value = config[i].f0
            biquadFilter.gain.value = config[i].db_gain
            biquadFilter.Q.value = config[i].q
            biquadFilter.getFrequencyResponse(frequency, magnitude, phaseResponse)
            for (var j = 0; j < STEPS; j += 1) {
              magnitudeSum[j] += magnitude[j]
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
      }, 20),
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