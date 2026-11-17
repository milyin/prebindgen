package nullability.negative
import io.prebindgen.covertest.nullability.*
fun requiredNulls() {
    nullabilityHandleRead(null) // REQUIRED_NULL
    nullabilityText(null) // REQUIRED_NULL
    nullabilityBytes(null) // REQUIRED_NULL
    nullabilityWide(null) // REQUIRED_NULL
    nullabilityTextCallback(null) // REQUIRED_NULL
    nullabilityTextCallback({ null }) // REQUIRED_NULL
    nullabilityValues(null) // REQUIRED_NULL
    nullabilityValues(listOf(null)) // REQUIRED_NULL
    nullabilityValuesOptional(listOf(null)) // REQUIRED_NULL
    nullabilityHandlesOptional(listOf(null)) // REQUIRED_NULL
}
