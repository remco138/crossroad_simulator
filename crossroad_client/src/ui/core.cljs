(ns ui.core
  (:require
   [figwheel.client :as fw :include-macros true]
   [reagent.core :as reagent :refer [atom]]
   [clojure.string :as string :refer [split-lines]]
   [cljsjs.paperjs]

   [network :as network]
   [state :as state]
   [drawing :as drawing]
   [cars :as cars]
   ))

(enable-console-print!)



(defn parse-svg [url]
  (let [item js/paper.project]
    (print (.importSVG item url))))

(defn spawn-car! []
  (let [car (state/add-car-channel! (cars/random-car　(:roads @state/state)))]
    (cars/car-ai (:chan car) car 0)))

(defn root-component []
  [:div
   [:canvas {:id "mycanvas" :width "967" :height "459"}]
   [:button  {:on-click spawn-car!} "yeee boiii. add a car"]
   [:button  {:on-click network/connect!} "(re)connect"]
  ])

(reagent/render
  [root-component]
  (.-body js/document))


(defn init! []
  (js/paper.setup(js/document.getElementById "mycanvas"))
  (set! js/paper.view.onFrame (var drawing/on-frame))
  (network/connect! 9991)
  (state/init!)
  (drawing/init!))

(set! (.-onload js/window) (fn [] (init!)))


(fw/watch-and-reload
  :websocket-url   "ws://localhost:3449/figwheel-ws"
  :jsload-callback (fn []
                     (init!)
                     (print "reloaded")))

