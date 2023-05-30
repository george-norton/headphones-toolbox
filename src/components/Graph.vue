<template>
  <Line :data="chartData" :options="options" style="background-image:url('src/assets/graph_bg.svg');background-repeat: no-repeat; background-size:contain"/>
</template>
     
<script>
import { Line } from 'vue-chartjs'
import { ref, reactive } from 'vue'
import { getCssVar } from 'quasar'
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

const frequency = [1, 16, 60, 250, 500, 2000, 4000, 6000, 20000];
const magnitude = [0, 16, 60, 250, 500, 2000, 4000, 6000, 20000];

export default {
  name: 'LineChart',
  components: { Line },
  data() {
    return {
      chartData: reactive({
        labels: frequency,
        datasets: [
          {
            label: "title",
            borderColor: getCssVar('primary'),
            data: magnitude,
            stepped: false,
            tension: 0
          }
        ]
      }),
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
                //    minRotation: 0
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