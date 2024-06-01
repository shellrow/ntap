import { reactive } from 'vue';

interface WindowSize {
  innerWidth: number;
  innerHeight: number;
}

export class WindowUtil {
  windowSize = reactive<WindowSize>({
    innerWidth: window.innerWidth,
    innerHeight: window.innerHeight,
  });

  private checkWindowSize = () => {
    this.windowSize = {
      innerWidth: window.innerWidth,
      innerHeight: window.innerHeight,
    };
  };

  mount() {
    window.addEventListener('resize', this.checkWindowSize);
  }

  unmount() {
    window.removeEventListener('resize', this.checkWindowSize);
  }
}
