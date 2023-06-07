<template>
  <Line :data="chartData" :options="options"
    style="background-image:url('graph_bg.svg'); background-repeat: no-repeat; background-size:contain" />
</template>
     
<script>
import { Line } from 'vue-chartjs'
import { ref, reactive } from 'vue'
import { getCssVar } from 'quasar'
import debounce from 'lodash.debounce'
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
      }, 50),
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
            borderWidth: 2
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
              //maxRotation: 0,
              //minRotation: 0
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